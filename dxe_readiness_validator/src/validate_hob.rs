use common::serializable_hob::HobSerDe;

pub fn validate_hob(hob_list: &[HobSerDe]) -> Result<(), String> {
    if hob_list.is_empty() {
        return Err("HOB list is empty".to_string());
    }

    Ok(())
}
