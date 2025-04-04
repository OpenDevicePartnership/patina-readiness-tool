// Clippy runs with "--all-targets," which includes "--tests." This module is
// marked to allow dead code to suppress Clippy warnings. Remove this once
// enough tests have been added.
#![allow(dead_code)]
#![allow(unused)]

use core::{ffi::c_void, mem, str};

use common::serializable_fv::FirmwareVolumeSerDe;
use common::serializable_hob::HobListSerDe;
use common::DxeReadinessCaptureSerDe;
use mu_pi::fw_fs::FirmwareVolume;
use mu_pi::hob::{header, Hob, HobList, PhaseHandoffInformationTable, HANDOFF};

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub(crate) fn read_phit_hob(physical_hob_list: *const c_void) -> Option<(usize, usize)> {
    if physical_hob_list.is_null() {
        panic!("HOB list pointer is null!");
    }

    let hob_header: *const header::Hob = physical_hob_list as *const header::Hob;

    // The PHIT HOB should always be first
    let current_header = unsafe { hob_header.cast::<header::Hob>().as_ref().expect(NOT_NULL) };
    if current_header.r#type == HANDOFF {
        assert_hob_size::<PhaseHandoffInformationTable>(current_header);
        let phit_hob = unsafe { hob_header.cast::<PhaseHandoffInformationTable>().as_ref().expect(NOT_NULL) };
        return Some((phit_hob.free_memory_bottom as usize, phit_hob.free_memory_top as usize));
    }

    None
}

pub(crate) const NOT_NULL: &str = "Ptr should not be NULL";

pub(crate) fn assert_hob_size<T>(hob: &header::Hob) {
    let hob_len = hob.length as usize;
    let hob_size = mem::size_of::<T>();
    assert_eq!(hob_len, hob_size, "Trying to cast hob of length {hob_len} into a pointer of size {hob_size}");
}

pub(crate) fn dump(hob_list: &HobList) -> Option<String> {
    let serializable_hob_list = HobListSerDe::from(hob_list);
    let serializable_fv_list = hob_list
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

    let serializable_list = DxeReadinessCaptureSerDe { hob_list: serializable_hob_list, fv_list: serializable_fv_list };
    serde_json::to_string_pretty(&serializable_list).ok()
}
