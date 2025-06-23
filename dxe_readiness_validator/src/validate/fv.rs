use crate::{
    validation_kind::{FvValidationKind, ValidationKind},
    validation_report::ValidationReport,
    validator::Validator,
    ValidationAppError,
};
use common::{format_guid, serializable_fv::FirmwareVolumeSerDe};
use r_efi::efi::Guid;

use super::ValidationResult;

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
    pub(super) fn validate_fv_for_traditional_smm(&self) -> ValidationResult {
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
    pub(super) fn validate_fv_for_combined_drivers(&self) -> ValidationResult {
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
    pub(super) fn validate_fv_for_apriori_file(&self) -> ValidationResult {
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

    /// Validates firmware volumes for sections compressed with LZMA and reports
    /// violations if any are found.
    pub(super) fn validate_fv_for_lzma_sections(&self) -> ValidationResult {
        let mut validation_report = ValidationReport::new();

        for fv in self.fv_list {
            for file in &fv.files {
                for section in &file.sections {
                    if section.compression_type.starts_with("LZMA ") {
                        validation_report.add_violation(ValidationKind::Fv(FvValidationKind::LzmaCompressedSections {
                            fv,
                            file,
                            section,
                        }));
                    }
                }
            }
        }

        Ok(validation_report)
    }
}

impl Validator for FvValidator<'_> {
    fn validate(&self) -> ValidationResult {
        let mut validation_report = ValidationReport::new();
        if self.fv_list.is_empty() {
            return Err(ValidationAppError::EmptyFvList);
        }

        validation_report.append_report(self.validate_fv_for_traditional_smm()?);
        validation_report.append_report(self.validate_fv_for_combined_drivers()?);
        validation_report.append_report(self.validate_fv_for_lzma_sections()?);
        validation_report.append_report(self.validate_fv_for_apriori_file()?);
        Ok(validation_report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::serializable_fv::FirmwareFileSerDe;
    use common::serializable_fv::FirmwareSectionSerDe;
    use common::serializable_fv::FirmwareVolumeSerDe;

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
                file_type: "CombinedPeimDriver".to_string(),
                length: 512,
                attributes: 0,
                sections: vec![FirmwareSectionSerDe {
                    section_type: "LZMA".to_string(),
                    length: 256,
                    compression_type: "LZMA ".to_string(),
                }],
            }],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_for_lzma_sections();
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
                file_type: "MmCore".to_string(),
                length: 128,
                attributes: 0,
                sections: vec![FirmwareSectionSerDe {
                    section_type: "LZMA".to_string(),
                    length: 128,
                    compression_type: "uncompressed".to_string(),
                }],
            }],
        }];

        let validator = FvValidator::new(&fv_list);
        let result = validator.validate_fv_for_lzma_sections();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
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
