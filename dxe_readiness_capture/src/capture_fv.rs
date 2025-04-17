use alloc::string::String;
use alloc::vec::Vec;
use common::serializable_fv::FirmwareVolumeSerDe;
use mu_pi::{
    fw_fs::FirmwareVolume,
    hob::{Hob, HobList},
};

pub(crate) fn capture_fv(hob_list: &HobList) -> Result<Vec<FirmwareVolumeSerDe>, String> {
    let fv_list: Vec<FirmwareVolumeSerDe> = hob_list
        .iter()
        .filter_map(|hob| {
            if let Hob::FirmwareVolume(&fv) = hob {
                let mut fv_serde =
                    FirmwareVolumeSerDe::from(unsafe { FirmwareVolume::new_from_address(fv.base_address) }.unwrap());
                fv_serde.fv_base_address = fv.base_address;
                Some(fv_serde)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(fv_list)
}
