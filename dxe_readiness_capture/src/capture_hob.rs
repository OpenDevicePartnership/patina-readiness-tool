use alloc::string::String;
use alloc::vec::Vec;
use common::serializable_hob::HobSerDe;
use mu_pi::hob::HobList;

pub(crate) fn capture_hob(hob_list: &HobList) -> Result<Vec<HobSerDe>, String> {
    let fv_list: Vec<HobSerDe> = hob_list.iter().map(HobSerDe::from).collect();
    Ok(fv_list)
}
