use common::serializable_fv::FirmwareVolumeSerDe;

pub fn validate_fv(fv_list: &[FirmwareVolumeSerDe]) -> Result<(), String> {
    if fv_list.is_empty() {
        return Err("FV list is empty".to_string());
    }

    Ok(())
}
