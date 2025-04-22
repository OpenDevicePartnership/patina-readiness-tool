use crate::{validate_fv::validate_fv, validate_hob::validate_hob};
use colored::*;
use common::DxeReadinessCaptureSerDe;
use std::collections::HashMap;
use std::fmt;
use std::fs;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ValidationKind {
    HobOverlappingMemoryRanges,
    TraditionalSmm,
    ProhibitedCombinedDrivers,
    ProhibitedAprioriFile,
    LzmaCompressedSections,
    MissingMemoryProtectionHob,
    InconsistentMemoryAttributes,
    V1MemoryRangeNotCotainnedInV2,
}

impl fmt::Display for ValidationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ValidationKind::HobOverlappingMemoryRanges => "HOB Overlapping Memory Ranges",
            ValidationKind::TraditionalSmm => "Traditional SMM",
            ValidationKind::ProhibitedCombinedDrivers => "Prohibited Combined Drivers",
            ValidationKind::ProhibitedAprioriFile => "Prohibited Apriori File",
            ValidationKind::LzmaCompressedSections => "LZMA Compressed Sections",
            ValidationKind::MissingMemoryProtectionHob => "Missing Memory Protection HOB",
            ValidationKind::InconsistentMemoryAttributes => "Inconsistent Memory Attributes",
            ValidationKind::V1MemoryRangeNotCotainnedInV2 => "V1 Memory Range Not Contained In V2",
        };
        write!(f, "{}", text)
    }
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    violations: HashMap<ValidationKind, Vec<String>>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self { violations: HashMap::new() }
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
                    writeln!(f, "#{}\n\t{}", j + 1, msg.replace("\n", "\n\t"))?;
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn validate(file_path: &String) -> Result<(), String> {
    let Ok(file_content) = fs::read_to_string(file_path) else {
        return Err(format!("Failed to read the file {}", file_path));
    };

    let Ok(json_data) = serde_json::from_str::<DxeReadinessCaptureSerDe>(&file_content) else {
        return Err(format!("Failed to parse JSON data from the file: {}", file_path));
    };

    let mut validation_report = ValidationReport::new();

    validate_hob(&json_data.hob_list, &mut validation_report)?;
    validate_fv(&json_data.fv_list, &mut validation_report)?;

    if !validation_report.is_empty() {
        log::info!("{}", validation_report);
    }

    Ok(())
}
