//! Validation logic for Firmware Volume (FV) structures.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use super::ValidationResult;
use crate::{
    ValidationAppError,
    validation_kind::{FvValidationKind, ValidationKind},
    validation_report::ValidationReport,
    validator::Validator,
};
use goblin::pe::{header::COFF_MACHINE_ARM64, subsystem::IMAGE_SUBSYSTEM_EFI_RUNTIME_DRIVER};
use patina::{
    base::UEFI_PAGE_SIZE,
    pi::serializable::{format_guid, serializable_fv::FirmwareVolumeSerDe},
};
use r_efi::efi::Guid;

/// Performs validation on a list of firmware volumes to check for violations of
/// Patina requirements.
pub struct FvValidator<'a> {
    fv_list: &'a Vec<FirmwareVolumeSerDe>,
}

impl<'a> FvValidator<'a> {
    pub fn new(fv_list: &'a Vec<FirmwareVolumeSerDe>) -> Self {
        FvValidator { fv_list }
    }

    /// Checks firmware volumes for files that use traditional SMM types and
    /// reports violations if found.
    pub(super) fn validate_fv_for_traditional_smm(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();

        self.fv_list.iter().for_each(|fv| {
            fv.files.iter().for_each(|file| match file.file_type.as_str() {
                "CombinedPeimDriver" | "Mm" | "CombinedMmDxe" | "MmCore" => validation_report
                    .add_violation(ValidationKind::Fv(FvValidationKind::UsesTraditionalSmm { file, fv })),
                _ => (),
            });
        });

        Ok(validation_report)
    }

    /// Checks firmware volumes for presence of combined driver files and
    /// reports violations if any are found.
    pub(super) fn validate_fv_for_combined_drivers(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();

        self.fv_list.iter().for_each(|fv| {
            fv.files.iter().for_each(|file| match file.file_type.as_str() {
                "CombinedPeimDriver" | "CombinedMmDxe" => validation_report
                    .add_violation(ValidationKind::Fv(FvValidationKind::CombinedDriversPresent { file, fv })),
                _ => (),
            });
        });

        Ok(validation_report)
    }

    /// Checks firmware volumes for presence of prohibited Apriori files by
    /// their GUIDs and reports violations if found.
    pub(super) fn validate_fv_for_apriori_file(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();

        let pei_apriori_file_name_guid = format_guid(Guid::from_fields(
            0x1B45CC0A,
            0x156A,
            0x428A,
            0xAF,
            0x62,
            &[0x49, 0x86, 0x4D, 0xA0, 0xE6, 0xE6],
        ));
        let apriori_file_name_guid = format_guid(Guid::from_fields(
            0xFC510EE7,
            0xFFDC,
            0x11D4,
            0xBD,
            0x41,
            &[0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81],
        ));

        self.fv_list.iter().for_each(|fv| {
            fv.files.iter().for_each(|file| {
                if file.name == pei_apriori_file_name_guid || file.name == apriori_file_name_guid {
                    validation_report
                        .add_violation(ValidationKind::Fv(FvValidationKind::ProhibitedAprioriFile { file, fv }));
                }
            });
        });

        Ok(validation_report)
    }

    /// Validates sections within firmware volumes for LZMA compression.
    /// For PE images, validates that the section alignment is correct.
    /// Reports violations if any are found.
    pub(super) fn validate_fv_file_sections(&self) -> ValidationResult<'_> {
        const FV_ARM64_RUNTIME_DRIVER_ALIGNMENT: usize = 0x10000;
        let mut validation_report = ValidationReport::new();

        for fv in self.fv_list {
            // Only process files whose type matches any of the allowed types.
            const ELIGIBLE_MODULE_TYPES: &[&str] = &["Driver", "Application", "DxeCore"];
            for file in &fv.files {
                if !ELIGIBLE_MODULE_TYPES.contains(&file.file_type.as_str()) {
                    continue;
                }
                for section in &file.sections {
                    if section.compression_type.starts_with("LZMA ") {
                        validation_report.add_violation(ValidationKind::Fv(FvValidationKind::LzmaCompressedSections {
                            fv,
                            file,
                            section,
                        }));
                    }

                    if section.section_type == "Pe32"
                        && let Some(pe_header_info) = &section.pe_info
                    {
                        // ARM64 DXE_RUNTIME_DRIVER needs 64k alignment.
                        if pe_header_info.machine == COFF_MACHINE_ARM64
                            && pe_header_info.subsystem == IMAGE_SUBSYSTEM_EFI_RUNTIME_DRIVER
                            && !(pe_header_info.section_alignment as usize)
                                .is_multiple_of(FV_ARM64_RUNTIME_DRIVER_ALIGNMENT)
                        {
                            validation_report.add_violation(ValidationKind::Fv(
                                FvValidationKind::InvalidSectionAlignment {
                                    fv,
                                    file,
                                    section,
                                    required_alignment: FV_ARM64_RUNTIME_DRIVER_ALIGNMENT,
                                },
                            ));
                        }
                        // Other sections can be just page-aligned (4k).
                        else if pe_header_info.section_alignment == 0
                            || !(pe_header_info.section_alignment as usize).is_multiple_of(UEFI_PAGE_SIZE)
                        {
                            validation_report.add_violation(ValidationKind::Fv(
                                FvValidationKind::InvalidSectionAlignment {
                                    fv,
                                    file,
                                    section,
                                    required_alignment: UEFI_PAGE_SIZE,
                                },
                            ));
                        }
                    }
                }
            }
        }

        Ok(validation_report)
    }
}

impl Validator for FvValidator<'_> {
    fn validate(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        if self.fv_list.is_empty() {
            return Err(ValidationAppError::EmptyFvList);
        }

        validation_report.append_report(self.validate_fv_for_traditional_smm()?);
        validation_report.append_report(self.validate_fv_for_combined_drivers()?);
        validation_report.append_report(self.validate_fv_file_sections()?);
        validation_report.append_report(self.validate_fv_for_apriori_file()?);
        Ok(validation_report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use goblin::pe::{
        header::COFF_MACHINE_X86_64,
        subsystem::{IMAGE_SUBSYSTEM_EFI_BOOT_SERVICE_DRIVER, IMAGE_SUBSYSTEM_EFI_RUNTIME_DRIVER},
    };
    use patina::pi::serializable::serializable_fv::{FirmwareFileSerDe, FirmwareSectionSerDe, PeHeaderInfo};

    #[test]
    fn test_validate_fv_for_traditional_smm() {
        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: "FV1".to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![
                FirmwareFileSerDe {
                    name: "File1".to_string(),
                    file_type: "CombinedPeimDriver".to_string(),
                    length: 512,
                    attributes: 0,
                    sections: vec![],
                },
                FirmwareFileSerDe {
                    name: "File2".to_string(),
                    file_type: "Mm".to_string(),
                    length: 256,
                    attributes: 0,
                    sections: vec![],
                },
                FirmwareFileSerDe {
                    name: "File3".to_string(),
                    file_type: "CombinedMmDxe".to_string(),
                    length: 256,
                    attributes: 0,
                    sections: vec![],
                },
                FirmwareFileSerDe {
                    name: "File4".to_string(),
                    file_type: "MmCore".to_string(),
                    length: 256,
                    attributes: 0,
                    sections: vec![],
                },
            ],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_validate_fv_combined_drivers() {
        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: "FV1".to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![
                FirmwareFileSerDe {
                    name: "File1".to_string(),
                    file_type: "CombinedPeimDriver".to_string(),
                    length: 512,
                    attributes: 0,
                    sections: vec![],
                },
                FirmwareFileSerDe {
                    name: "File2".to_string(),
                    file_type: "CombinedMmDxe".to_string(),
                    length: 256,
                    attributes: 0,
                    sections: vec![],
                },
            ],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_for_combined_drivers();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);

        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: "FV2".to_string(),
            fv_length: 2048,
            fv_base_address: 0x2000,
            fv_attributes: 0,
            files: vec![
                FirmwareFileSerDe {
                    name: "File3".to_string(),
                    file_type: "Dxe".to_string(),
                    length: 128,
                    attributes: 0,
                    sections: vec![],
                },
                FirmwareFileSerDe {
                    name: "File4".to_string(),
                    file_type: "MmCore".to_string(),
                    length: 64,
                    attributes: 0,
                    sections: vec![],
                },
            ],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_for_combined_drivers();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_validate_fv_for_apriori_file() {
        let pei_apriori_file_name_guid = format_guid(Guid::from_fields(
            0x1B45CC0A,
            0x156A,
            0x428A,
            0xAF,
            0x62,
            &[0x49, 0x86, 0x4D, 0xA0, 0xE6, 0xE6],
        ));
        let apriori_file_name_guid = format_guid(Guid::from_fields(
            0xFC510EE7,
            0xFFDC,
            0x11D4,
            0xBD,
            0x41,
            &[0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81],
        ));

        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: "FV1".to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![FirmwareFileSerDe {
                name: pei_apriori_file_name_guid,
                file_type: "Dxe".to_string(),
                length: 512,
                attributes: 0,
                sections: vec![],
            }],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_for_apriori_file();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);

        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: "FV1".to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![FirmwareFileSerDe {
                name: apriori_file_name_guid,
                file_type: "Dxe".to_string(),
                length: 512,
                attributes: 0,
                sections: vec![],
            }],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_for_apriori_file();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_validate_fv_for_lzma_sections() {
        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: "FV1".to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![FirmwareFileSerDe {
                name: "File1".to_string(),
                file_type: "Driver".to_string(),
                length: 512,
                attributes: 0,
                sections: vec![FirmwareSectionSerDe {
                    section_type: "LZMA".to_string(),
                    length: 256,
                    compression_type: "LZMA ".to_string(),
                    pe_info: None,
                }],
            }],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_file_sections();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);

        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: "FV2".to_string(),
            fv_length: 2048,
            fv_base_address: 0x2000,
            fv_attributes: 0,
            files: vec![FirmwareFileSerDe {
                name: "File3".to_string(),
                file_type: "MmCoreStandalone".to_string(),
                length: 128,
                attributes: 0,
                sections: vec![FirmwareSectionSerDe {
                    section_type: "LZMA".to_string(),
                    length: 128,
                    compression_type: "uncompressed".to_string(),
                    pe_info: None,
                }],
            }],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_file_sections();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    // Helper function to run alignment validation and return violation count
    fn run_alignment_test(
        fv_name: &str,
        file_name: &str,
        file_type: &str,
        section_alignment: u32,
        machine: u16,
        subsystem: u16,
    ) -> usize {
        let fv_list = vec![FirmwareVolumeSerDe {
            fv_name: fv_name.to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![FirmwareFileSerDe {
                name: file_name.to_string(),
                file_type: file_type.to_string(),
                length: 512,
                attributes: 0,
                sections: vec![FirmwareSectionSerDe {
                    section_type: "Pe32".to_string(),
                    length: 256,
                    compression_type: "uncompressed ".to_string(),
                    pe_info: Some(PeHeaderInfo { section_alignment, machine, subsystem }),
                }],
            }],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_file_sections();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        validation_report.violation_count()
    }

    #[test]
    fn test_invalid_alignment_not_multiple_of_page_size() {
        let violation_count = run_alignment_test(
            "FV1",
            "File1",
            "Driver",
            12345, // Not a valid multiple of page size
            COFF_MACHINE_X86_64,
            IMAGE_SUBSYSTEM_EFI_RUNTIME_DRIVER,
        );
        assert_eq!(violation_count, 1);
    }

    #[test]
    fn test_invalid_alignment_zero() {
        let violation_count = run_alignment_test(
            "FV1",
            "File1",
            "Driver",
            0, // Zero alignment not allowed
            COFF_MACHINE_X86_64,
            IMAGE_SUBSYSTEM_EFI_BOOT_SERVICE_DRIVER,
        );
        assert_eq!(violation_count, 1);
    }

    #[test]
    fn test_valid_alignment_multiple_of_page_size() {
        let violation_count = run_alignment_test(
            "FV2",
            "File3",
            "MmCore",
            (UEFI_PAGE_SIZE * 2) as u32, // Valid multiple of page size
            COFF_MACHINE_X86_64,
            IMAGE_SUBSYSTEM_EFI_BOOT_SERVICE_DRIVER,
        );
        assert_eq!(violation_count, 0);
    }

    #[test]
    fn test_invalid_alignment_arm64_runtime_driver() {
        let violation_count = run_alignment_test(
            "FV2",
            "File3",
            "Driver",
            (UEFI_PAGE_SIZE * 2) as u32, // Valid multiple of page size but NOT valid for ARM64 DXE_RUNTIME_DRIVER
            COFF_MACHINE_ARM64,
            IMAGE_SUBSYSTEM_EFI_RUNTIME_DRIVER,
        );
        assert_eq!(violation_count, 1);
    }

    #[test]
    fn test_ineligible_modules_types_do_not_fail() {
        const ALL_MODULE_TYPES: &[&str] = &[
            "Raw",
            "FreeForm",
            "SecurityCore",
            "PeiCore",
            "DxeCore",
            "Peim",
            "Driver",
            "CombinedPeimDriver",
            "Application",
            "Mm",
            "FirmwareVolumeImage",
            "CombinedMmDxe",
            "MmCore",
            "MmStandalone",
            "MmCoreStandalone",
            "FfsPad",
            "FfsUnknown",
        ];
        const ELIGIBLE_MODULE_TYPES: &[&str] = &["Driver", "Application", "DxeCore"];

        for &module_type in ALL_MODULE_TYPES {
            let expected_violation = ELIGIBLE_MODULE_TYPES.contains(&module_type);
            let violation_count = run_alignment_test(
                "TestFv",
                "TestFile",
                module_type,
                UEFI_PAGE_SIZE as u32, // 4k aligned
                COFF_MACHINE_X86_64,
                IMAGE_SUBSYSTEM_EFI_BOOT_SERVICE_DRIVER,
            );
            if expected_violation {
                assert_eq!(violation_count, 0, "Eligible type '{}' should not violate", module_type);
            } else {
                assert_eq!(violation_count, 0, "Ineligible type '{}' should not violate", module_type);
            }
        }
    }

    #[test]
    fn test_validate_empty_list() {
        let fv_list = vec![];
        let validator = FvValidator::new(&fv_list);
        let result = validator.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationAppError::EmptyFvList);
    }
}
