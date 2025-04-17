use crate::{validate_fv::validate_fv, validate_hob::validate_hob};
use common::DxeReadinessCaptureSerDe;
use std::fs;

pub(crate) fn validate(file_path: &String) -> Result<(), String> {
    let Ok(file_content) = fs::read_to_string(file_path) else {
        return Err(format!("Failed to read the file {}", file_path));
    };

    let Ok(json_data) = serde_json::from_str::<DxeReadinessCaptureSerDe>(&file_content) else {
        return Err(format!("Failed to parse JSON data from the file: {}", file_path));
    };

    validate_hob(&json_data.hob_list)?;
    validate_fv(&json_data.fv_list)?;

    Ok(())
}
