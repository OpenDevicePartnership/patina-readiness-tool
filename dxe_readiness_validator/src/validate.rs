//! Main entry point and orchestration for running all DXE readiness validations.
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
mod fv;
mod hob;
use crate::commandline::CommandLine;
use crate::validation_report::ValidationReport;
use crate::validator::Validator;
use crate::ValidationAppError;
use clap::CommandFactory;
use clap::Parser;
use common::DxeReadinessCaptureSerDe;
use fv::FvValidator;
use hob::HobValidator;
use std::fs;

pub type ValidationResult<'a> = Result<ValidationReport<'a>, ValidationAppError>;

pub struct ValidationApp {
    args: CommandLine,
    data: Option<DxeReadinessCaptureSerDe>,
}

impl ValidationApp {
    pub fn new() -> Self {
        Self { args: CommandLine::parse(), data: None }
    }

    /// Parses a JSON file specified by the command-line arguments and populates
    /// the internal data.
    pub fn parse_json(&mut self) -> Result<(), ValidationAppError> {
        let Some(ref filename) = self.args.filename else {
            let _ = CommandLine::command().print_help();
            return Err(ValidationAppError::InvalidCommandLine("'filename'".to_string()));
        };

        let file_content = fs::read_to_string(filename).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                ValidationAppError::JSONFileNotFound(filename.clone())
            } else {
                ValidationAppError::JSONFileContentError(filename.clone(), err.to_string())
            }
        })?;

        let data = serde_json::from_str::<DxeReadinessCaptureSerDe>(&file_content)
            .map_err(|err| ValidationAppError::JSONSerializationFailed(filename.clone(), err.to_string()))?;

        self.data = Some(data);
        Ok(())
    }

    /// Validates the contents of the parsed JSON data, including HOBs and
    /// firmware volumes.
    pub fn validate(&self) -> Result<(), ValidationAppError> {
        let Some(data) = &self.data else {
            return Err(ValidationAppError::EmptyHobList);
        };

        let mut validation_report = ValidationReport::new();

        let hob_validator = HobValidator::new(&data.hob_list);
        validation_report.append_report(hob_validator.validate()?);

        let fv_validator = FvValidator::new(&data.fv_list);
        validation_report.append_report(fv_validator.validate()?);

        validation_report.show_results();

        let validation_count = validation_report.violation_count() as u32;
        if validation_count != 0 {
            return Err(ValidationAppError::ValidationErrors(validation_count));
        }

        Ok(())
    }
}
