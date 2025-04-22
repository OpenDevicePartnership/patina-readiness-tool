use std::collections::HashMap;

use crate::validate::{ValidationKind, ValidationReport};
use common::{
    format_guid,
    serializable_fv::{FirmwareFileSerDe, FirmwareVolumeSerDe},
};
use r_efi::efi::Guid;

pub fn validate_fv_standalone_mm(
    fv_list: &[FirmwareVolumeSerDe],
    validation_report: &mut ValidationReport,
) -> Result<(), String> {
    let mut fv_map: HashMap<String, Vec<FirmwareFileSerDe>> = HashMap::new();

    fv_list.iter().for_each(|fv| {
        fv.files.iter().for_each(|file| {
            if file.file_type == "CombinedPeimDriver"
                || file.file_type == "Mm"
                || file.file_type == "CombinedMmDxe"
                || file.file_type == "MmCore"
            {
                fv_map.entry(fv.fv_name.clone()).or_default().push(file.clone());
            }
        });
    });

    let mut validation_result = String::new();
    for (fv_name, files) in &fv_map {
        validation_result.push_str(&format!("FV Name: {}\n", fv_name));
        for file in files {
            if let Ok(json_str) = serde_json::to_string_pretty(file) {
                validation_result.push_str(&format!("File: {}\n", json_str));
            }
        }
    }

    if !validation_result.is_empty() {
        validation_report.add_violation(ValidationKind::TraditionalSmm, &validation_result);
    }

    Ok(())
}

pub fn validate_fv_combined_drivers(
    fv_list: &[FirmwareVolumeSerDe],
    validation_report: &mut ValidationReport,
) -> Result<(), String> {
    let mut fv_map: HashMap<String, Vec<FirmwareFileSerDe>> = HashMap::new();

    fv_list.iter().for_each(|fv| {
        fv.files.iter().for_each(|file| {
            if file.file_type == "CombinedPeimDriver" || file.file_type == "CombinedMmDxe" {
                fv_map.entry(fv.fv_name.clone()).or_default().push(file.clone());
            }
        });
    });

    let mut validation_result = String::new();
    for (fv_name, files) in &fv_map {
        validation_result.push_str(&format!("FV Name: {}\n", fv_name));
        for file in files {
            if let Ok(json_str) = serde_json::to_string_pretty(file) {
                validation_result.push_str(&format!("File: {}\n", json_str));
            }
        }
    }

    if !validation_result.is_empty() {
        validation_report.add_violation(ValidationKind::ProhibitedCombinedDrivers, &validation_result);
    }

    Ok(())
}

pub fn validate_fv_apriori_file(
    fv_list: &[FirmwareVolumeSerDe],
    validation_report: &mut ValidationReport,
) -> Result<(), String> {
    let pei_apriori_file_name_guid =
        format_guid(Guid::from_fields(0x1B45CC0A, 0x156A, 0x428A, 0xAF, 0x62, &[0x49, 0x86, 0x4D, 0xA0, 0xE6, 0xE6]));
    let apriori_file_name_guid =
        format_guid(Guid::from_fields(0xFC510EE7, 0xFFDC, 0x11D4, 0xBD, 0x41, &[0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81]));

    let mut fv_map: HashMap<String, Vec<FirmwareFileSerDe>> = HashMap::new();
    fv_list.iter().for_each(|fv| {
        fv.files.iter().for_each(|file| {
            if file.name == pei_apriori_file_name_guid || file.name == apriori_file_name_guid {
                fv_map.entry(fv.fv_name.clone()).or_default().push(file.clone());
            }
        });
    });

    let mut validation_result = String::new();
    for (fv_name, files) in &fv_map {
        validation_result.push_str(&format!("FV Name: {}\n", fv_name));
        for file in files {
            if let Ok(json_str) = serde_json::to_string_pretty(file) {
                validation_result.push_str(&format!("File: {}\n", json_str));
            }
        }
    }

    if !validation_result.is_empty() {
        validation_report.add_violation(ValidationKind::ProhibitedAprioriFile, &validation_result);
    }

    Ok(())
}

pub fn validate_fv_file_sections(
    fv_list: &[FirmwareVolumeSerDe],
    validation_report: &mut ValidationReport,
) -> Result<(), String> {
    let mut fv_map: HashMap<String, Vec<FirmwareFileSerDe>> = HashMap::new();

    for fv in fv_list {
        for file in &fv.files {
            for section in &file.sections {
                if section.compression_type.starts_with("LZMA ") {
                    fv_map.entry(fv.fv_name.clone()).or_default().push(file.clone());
                    break; // Only need to check the first section for LZMA compression
                }
            }
        }
    }

    let mut validation_result = String::new();
    for (fv_name, files) in &fv_map {
        validation_result.push_str(&format!("FV Name: {}\n", fv_name));
        for file in files {
            validation_result.push_str(&format!("File Name: {}\n", file.name));
            for section in &file.sections {
                if section.compression_type.starts_with("LZMA ") {
                    if let Ok(json_str) = serde_json::to_string_pretty(section) {
                        validation_result.push_str(&format!("Section: {}\n", json_str));
                    }
                }
            }
        }
    }

    if !validation_result.is_empty() {
        validation_report.add_violation(ValidationKind::LzmaCompressedSections, &validation_result);
    }

    Ok(())
}

pub fn validate_fv(fv_list: &[FirmwareVolumeSerDe], validation_report: &mut ValidationReport) -> Result<(), String> {
    if fv_list.is_empty() {
        return Err("FV list is empty".to_string());
    }

    validate_fv_standalone_mm(fv_list, validation_report)?;
    validate_fv_combined_drivers(fv_list, validation_report)?;
    validate_fv_file_sections(fv_list, validation_report)?;
    validate_fv_apriori_file(fv_list, validation_report)?;
    Ok(())
}
