mod fv;
mod hob;
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
pub enum HobValidationKind {
    InconsistentMemoryAttributes,        // HOBs must define consistent memory attributes
    OverlappingMemoryRanges,             // HOBs must not define overlapping memory ranges
    PageZeroMemoryDescribed,             // Page zero must not be described in memory HOBs
    V1MemoryRangeNotContainedInV2,       // All V1 ranges must be covered by V2
    V2ContainsUceAttribute,              // V2 ranges must not have the UCE attribute
    V2MissingValidCacheabilityAttribute, // V2 resource descriptor must have at least one valid Cacheability attribute set
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FvValidationKind {
    CombinedDriversPresent, // FV must not contain combined drivers
    LzmaCompressedSections, // FV must not contain LZMA-compressed sections
    ProhibitedAprioriFile,  // FV must not contain an Apriori file
    UsesTraditionalSmm,     // FV must not contain traditional SMM drivers
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationKind {
    Hob(HobValidationKind),
    Fv(FvValidationKind),
}

impl fmt::Display for ValidationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ValidationKind::Hob(hob_validation_kind) => match hob_validation_kind {
                HobValidationKind::InconsistentMemoryAttributes => "HOBs with inconsistent memory attributes",
                HobValidationKind::OverlappingMemoryRanges => "HOBs with overlapping memory ranges",
                HobValidationKind::PageZeroMemoryDescribed => "HOB describing page zero memory allocation",
                HobValidationKind::V1MemoryRangeNotContainedInV2 => "V1 memory range not contained within V2",
                HobValidationKind::V2ContainsUceAttribute => "V2 HOB contains prohibited EFI_MEMORY_UCE attribute",
                HobValidationKind::V2MissingValidCacheabilityAttribute => {
                    "V2 HOB does not contain a valid cacheability attribute."
                }
            },
            ValidationKind::Fv(fv_validation_kind) => match fv_validation_kind {
                FvValidationKind::CombinedDriversPresent => "Firmware volume contains prohibited combined drivers",
                FvValidationKind::LzmaCompressedSections => "Firmware volume contains LZMA-compressed sections",
                FvValidationKind::ProhibitedAprioriFile => "Firmware volume contains a prohibited APRIORI file",
                FvValidationKind::UsesTraditionalSmm => "Firmware volume contains traditional SMM drivers",
            },
        };
        write!(f, "{}", text)
    }
}

impl ValidationKind {
    pub fn guidance(&self) -> &str {
        match self {
            ValidationKind::Hob(hob_validation_kind) => match hob_validation_kind {
                HobValidationKind::InconsistentMemoryAttributes => "V1 and V2 HOBs describing the same range(s) with inconsistent memory attributes are not supported.",
                HobValidationKind::OverlappingMemoryRanges => "HOBs with overlapping memory ranges are not supported.",
                HobValidationKind::PageZeroMemoryDescribed => "HOB describing page zero memory allocation not supported. As page zero will be used to detect null pointer dereferences",
                HobValidationKind::V1MemoryRangeNotContainedInV2 => "All V1 HOB ranges should be described/covered by corresponding V2 HOBs.",
                HobValidationKind::V2ContainsUceAttribute => "V2 HOB contains prohibited EFI_MEMORY_UCE attribute.",
                HobValidationKind::V2MissingValidCacheabilityAttribute => "V2 resource descriptor must have atleast one valid Cacheability attribute set\n- MEMORY_UC\n- MEMORY_UCE\n- MEMORY_WB\n- MEMORY_WC\n- MEMORY_WP\n- MEMORY_WT\n",
            },
            ValidationKind::Fv(fv_validation_kind) => match fv_validation_kind {
                FvValidationKind::CombinedDriversPresent => "Firmware volume contains prohibited combined drivers. \nBelow file types are prohibited\n- COMBINED_MM_DXE(0x0C)\n- COMBINED_PEIM_DRIVER(0x08).",
                FvValidationKind::LzmaCompressedSections => "Firmware volume contains LZMA-compressed sections. Rust Dxe Core do not have support for LZMA compression.",
                FvValidationKind::ProhibitedAprioriFile => "Firmware volume contains a prohibited A priori file. Rust Dxe Core do not support A priori based driver dispatch.",
                FvValidationKind::UsesTraditionalSmm => "Firmware volume contains traditional SMM drivers. Below file types are prohibited\n- COMBINED_MM_DXE(0x0C)\n- COMBINED_PEIM_DRIVER(0x08)\n- MM(0x0A)\n- MM_CORE(0x0D).",
            },
        }
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
            writeln!(f, "{}", "Validation Results:".red().bold())?;
            for (kind, messages) in &self.violations {
                if messages.is_empty() {
                    continue;
                }
                writeln!(f, "──────────────────────────────────────────────────────────────────")?;
                writeln!(f, "{} ❌\n", format!("- {}", kind).green().bold())?;
                for (j, msg) in messages.iter().enumerate() {
                    writeln!(f, "{}. {}", j + 1, msg)?;
                }
                writeln!(f, "\n💡 {}", format!("Guidance:\n{}", kind.guidance()).blue().bold())?;
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

    pub fn validate(&mut self) -> Result<usize, String> {
        self.deserialize_json()?;
        self.validate_hobs()?;
        self.validate_firmware_volumes()?;

        if !self.validation_report.is_empty() {
            log::info!("{}", self.validation_report);
        } else {
            log::info!("Validation passed with no errors.");
        }

        Ok(self.validation_report.violations.len())
    }
}
