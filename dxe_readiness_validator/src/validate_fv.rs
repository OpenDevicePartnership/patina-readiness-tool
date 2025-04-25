use crate::{
    validate::{ValidationApp, ValidationKind},
    ValidationResult,
};
use common::{format_guid, DxeReadinessCaptureSerDe};
use r_efi::efi::Guid;

impl ValidationApp {
    fn validate_fv_standalone_mm(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref fv_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        fv_list.iter().for_each(|fv| {
            fv.files.iter().for_each(|file| {
                if file.file_type == "CombinedPeimDriver"
                    || file.file_type == "Mm"
                    || file.file_type == "CombinedMmDxe"
                    || file.file_type == "MmCore"
                {
                    if let Ok(json_str) = serde_json::to_string_pretty(file) {
                        self.validation_report.add_violation(
                            ValidationKind::TraditionalSmm,
                            &format!("FV: {} File: {}", fv.fv_name, json_str),
                        );
                    }
                }
            });
        });

        Ok(())
    }

    fn validate_fv_combined_drivers(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref fv_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        fv_list.iter().for_each(|fv| {
            fv.files.iter().for_each(|file| {
                if file.file_type == "CombinedPeimDriver" || file.file_type == "CombinedMmDxe" {
                    if let Ok(json_str) = serde_json::to_string_pretty(file) {
                        self.validation_report.add_violation(
                            ValidationKind::ProhibitedCombinedDrivers,
                            &format!("FV: {} File: {}", fv.fv_name, json_str),
                        );
                    }
                }
            });
        });

        Ok(())
    }

    fn validate_fv_apriori_file(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref fv_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

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

        fv_list.iter().for_each(|fv| {
            fv.files.iter().for_each(|file| {
                if file.name == pei_apriori_file_name_guid || file.name == apriori_file_name_guid {
                    if let Ok(json_str) = serde_json::to_string_pretty(file) {
                        self.validation_report.add_violation(
                            ValidationKind::ProhibitedAprioriFile,
                            &format!("FV: {} File: {}", fv.fv_name, json_str),
                        );
                    }
                }
            });
        });

        Ok(())
    }

    fn validate_fv_file_sections(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref fv_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        for fv in fv_list {
            for file in &fv.files {
                for section in &file.sections {
                    if section.compression_type.starts_with("LZMA ") {
                        if let Ok(json_str) = serde_json::to_string_pretty(section) {
                            self.validation_report.add_violation(
                                ValidationKind::LzmaCompressedSections,
                                &format!("FV: {} File: {} Section: {}", fv.fv_name, file.name, json_str),
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn validate_firmware_volumes(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref fv_list, .. }) = self.data.as_ref() else {
            return Err("FV list is empty".to_string());
        };

        if fv_list.is_empty() {
            return Err("FV list is empty".to_string());
        }

        self.validate_fv_standalone_mm()?;
        self.validate_fv_combined_drivers()?;
        self.validate_fv_file_sections()?;
        self.validate_fv_apriori_file()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::serializable_fv::FirmwareFileSerDe;
    use common::serializable_fv::FirmwareSectionSerDe;
    use common::serializable_fv::FirmwareVolumeSerDe;

    #[test]
    fn test_validate_fv_standalone_mm() {
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

        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_firmware_volumes();
        assert!(result.is_ok());
        assert!(!app.validation_report.is_empty());
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

        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_fv_combined_drivers();
        assert!(result.is_ok());
        assert!(!app.validation_report.is_empty());

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

        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_fv_combined_drivers();
        assert!(result.is_ok());
        assert!(app.validation_report.is_empty());
    }

    #[test]
    fn test_validate_fv_apriori_file() {
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

        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_fv_apriori_file();
        assert!(result.is_ok());
        assert!(!app.validation_report.is_empty());

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

        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_fv_apriori_file();
        assert!(result.is_ok());
        assert!(!app.validation_report.is_empty());
    }

    #[test]
    fn test_validate_fv_file_sections() {
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

        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_fv_file_sections();
        assert!(result.is_ok());
        assert!(!app.validation_report.is_empty());

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

        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_fv_file_sections();
        assert!(result.is_ok());
        assert!(app.validation_report.is_empty());
    }

    #[test]
    fn test_validate_fv_empty_list() {
        let data = DxeReadinessCaptureSerDe { hob_list: vec![], fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let result = app.validate_firmware_volumes();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "FV list is empty".to_string());
    }
}
