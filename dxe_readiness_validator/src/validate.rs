//! Main entry point and orchestration for running all DXE readiness validations.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
mod fv;
mod hob;
use crate::{ValidationAppError, commandline::CommandLine, validation_report::ValidationReport, validator::Validator};
use clap::{CommandFactory, Parser};
use dxe_readiness_capture::DxeReadinessCaptureSerDe;
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
        validation_report.add_summary(hob_validator.summary());
        validation_report.append_report(hob_validator.validate()?);

        let fv_validator = FvValidator::new(&data.fv_list);
        validation_report.add_summary(fv_validator.summary());
        validation_report.append_report(fv_validator.validate()?);

        validation_report.show_results();

        let validation_count = validation_report.violation_count() as u32;
        if validation_count != 0 {
            return Err(ValidationAppError::ValidationErrors(validation_count));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use patina::pi::serializable::{
        serializable_fv::FirmwareVolumeSerDe,
        serializable_hob::{HobSerDe, ResourceDescriptorSerDe},
    };
    use r_efi::efi;

    fn app_with_data(data: Option<DxeReadinessCaptureSerDe>) -> ValidationApp {
        ValidationApp { args: CommandLine { filename: None }, data }
    }

    fn app_with_file(filename: Option<String>) -> ValidationApp {
        ValidationApp { args: CommandLine { filename }, data: None }
    }

    fn temp_path(name: &str) -> String {
        let mut path = std::env::temp_dir();
        path.push(format!("dxe_val_test_{}_{name}", std::process::id()));
        path.to_string_lossy().to_string()
    }

    /// A V2 resource descriptor with a valid single cacheability attribute that
    /// passes all HOB checks.
    fn clean_v2_hob() -> HobSerDe {
        HobSerDe::ResourceDescriptorV2 {
            v1: ResourceDescriptorSerDe {
                owner: "owner".to_string(),
                resource_type: 3,
                resource_attribute: 0,
                physical_start: 0x100000,
                resource_length: 0x1000,
            },
            attributes: efi::MEMORY_UC,
        }
    }

    /// A lone V1 resource descriptor with no covering V2, which triggers exactly
    /// one violation.
    fn lone_v1_hob() -> HobSerDe {
        HobSerDe::ResourceDescriptor(ResourceDescriptorSerDe {
            owner: "owner".to_string(),
            resource_type: 3,
            resource_attribute: 0,
            physical_start: 0x100000,
            resource_length: 0x1000,
        })
    }

    fn clean_fv() -> FirmwareVolumeSerDe {
        FirmwareVolumeSerDe {
            fv_name: "FvMain".to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![],
        }
    }

    #[test]
    fn test_validate_none_data_errors() {
        let app = app_with_data(None);
        assert_eq!(app.validate().unwrap_err(), ValidationAppError::EmptyHobList);
    }

    #[test]
    fn test_validate_clean_data_ok() {
        let data = DxeReadinessCaptureSerDe { hob_list: vec![clean_v2_hob()], fv_list: vec![clean_fv()] };
        let app = app_with_data(Some(data));
        assert!(app.validate().is_ok());
    }

    #[test]
    fn test_validate_reports_violations() {
        let data = DxeReadinessCaptureSerDe { hob_list: vec![lone_v1_hob()], fv_list: vec![clean_fv()] };
        let app = app_with_data(Some(data));
        assert_eq!(app.validate().unwrap_err(), ValidationAppError::ValidationErrors(1));
    }

    #[test]
    fn test_validate_empty_hob_list_propagates() {
        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list: vec![clean_fv()] };
        let app = app_with_data(Some(data));
        assert_eq!(app.validate().unwrap_err(), ValidationAppError::EmptyHobList);
    }

    #[test]
    fn test_validate_empty_fv_list_propagates() {
        let data = DxeReadinessCaptureSerDe { hob_list: vec![clean_v2_hob()], fv_list: vec![] };
        let app = app_with_data(Some(data));
        assert_eq!(app.validate().unwrap_err(), ValidationAppError::EmptyFvList);
    }

    #[test]
    fn test_parse_json_missing_filename_errors() {
        let mut app = app_with_file(None);
        assert!(matches!(app.parse_json(), Err(ValidationAppError::InvalidCommandLine(_))));
    }

    #[test]
    fn test_parse_json_file_not_found() {
        let mut app = app_with_file(Some(temp_path("does_not_exist.json")));
        assert!(matches!(app.parse_json(), Err(ValidationAppError::JSONFileNotFound(_))));
    }

    #[test]
    fn test_parse_json_invalid_content() {
        let path = temp_path("invalid.json");
        fs::write(&path, "not valid json").unwrap();
        let mut app = app_with_file(Some(path.clone()));
        let result = app.parse_json();
        let _ = fs::remove_file(&path);
        assert!(matches!(result, Err(ValidationAppError::JSONSerializationFailed(..))));
    }

    #[test]
    fn test_parse_json_valid_content() {
        let path = temp_path("valid.json");
        fs::write(&path, r#"{"hob_list": [], "fv_list": []}"#).unwrap();
        let mut app = app_with_file(Some(path.clone()));
        let result = app.parse_json();
        let _ = fs::remove_file(&path);
        assert!(result.is_ok());
        assert!(app.data.is_some());
    }
}
