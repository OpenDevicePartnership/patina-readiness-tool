use crate::commandline::CommandLine;
use crate::ValidationResult;
use clap::CommandFactory;
use clap::Parser;
use colored::*;
use common::DxeReadinessCaptureSerDe;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationKind {
    HobOverlappingMemoryRanges,
    InconsistentMemoryAttributes,
    LzmaCompressedSections,
    MissingMemoryProtectionHob,
    ProhibitedAprioriFile,
    ProhibitedCombinedDrivers,
    TraditionalSmm,
    V1MemoryRangeNotCotainnedInV2,
}

impl fmt::Display for ValidationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ValidationKind::HobOverlappingMemoryRanges => "HOB Overlapping Memory Ranges",
            ValidationKind::InconsistentMemoryAttributes => "Inconsistent Memory Attributes",
            ValidationKind::LzmaCompressedSections => "LZMA Compressed Sections",
            ValidationKind::MissingMemoryProtectionHob => "Missing Memory Protection HOB",
            ValidationKind::ProhibitedAprioriFile => "Prohibited Apriori File",
            ValidationKind::ProhibitedCombinedDrivers => "Prohibited Combined Drivers",
            ValidationKind::TraditionalSmm => "Traditional SMM",
            ValidationKind::V1MemoryRangeNotCotainnedInV2 => "V1 Memory Range Not Contained In V2",
        };
        write!(f, "{}", text)
    }
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    violations: BTreeMap<ValidationKind, Vec<String>>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self { violations: BTreeMap::new() }
    }

    pub fn add_violation(&mut self, validation: ValidationKind, message: &str) {
        self.violations.entry(validation).or_default().push(message.to_string());
    }

    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.violations.is_empty() {
            writeln!(f, "No validation violations found.")?;
        } else {
            writeln!(f, "{}", "Validation violations:".red().bold())?;
            for (i, (kind, messages)) in self.violations.iter().enumerate() {
                if messages.is_empty() {
                    continue;
                }
                writeln!(f, "{}", format!("{}. {}:", i + 1, kind).green().bold())?;
                for (j, msg) in messages.iter().enumerate() {
                    writeln!(f, "{}. {}", j + 1, msg)?;
                }
            }
        }
        Ok(())
    }
}

pub struct ValidationApp {
    args: CommandLine,
    pub validation_report: ValidationReport,
    pub data: Option<DxeReadinessCaptureSerDe>,
}

impl ValidationApp {
    pub fn new() -> Self {
        Self { args: CommandLine::parse(), validation_report: ValidationReport::new(), data: None }
    }

    #[cfg(test)]
    pub fn new_with_data(data: DxeReadinessCaptureSerDe) -> Self {
        Self { args: CommandLine::default(), validation_report: ValidationReport::new(), data: Some(data) }
    }

    fn deserialize_json(&mut self) -> ValidationResult {
        if self.data.is_some() {
            return Ok(());
        }

        let Some(ref filename) = self.args.filename else {
            return CommandLine::command().print_help().map_err(|e| e.to_string());
        };

        let Ok(file_content) = fs::read_to_string(filename) else {
            return Err(format!("Failed to read the file {}", filename));
        };

        let Ok(data) = serde_json::from_str::<DxeReadinessCaptureSerDe>(&file_content) else {
            return Err(format!("Failed to parse JSON data from the file: {}", filename));
        };

        self.data = Some(data);
        Ok(())
    }

    pub fn validate(&mut self) -> ValidationResult {
        self.deserialize_json()?;
        self.validate_hobs()?;
        self.validate_firmware_volumes()?;

        if !self.validation_report.is_empty() {
            log::info!("{}", self.validation_report);
        }

        Ok(())
    }
}
