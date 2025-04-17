// Clippy runs with "--all-targets," which includes "--tests." This module is
// marked to allow dead code to suppress Clippy warnings. Remove this once
// enough tests have been added.
#![allow(dead_code)]
#![allow(unused)]

use core::{ffi::c_void, mem, str};

use common::serializable_fv::FirmwareVolumeSerDe;
use common::serializable_hob::HobSerDe;
use common::DxeReadinessCaptureSerDe;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use mu_pi::hob::HobList;

use crate::capture_fv;
use crate::capture_hob;

pub(crate) fn capture(hob_list: &HobList) -> Result<String, String> {
    let serializable_hob_list: Vec<HobSerDe> = capture_hob::capture_hob(hob_list)?;
    let serializable_fv_list: Vec<FirmwareVolumeSerDe> = capture_fv::capture_fv(hob_list)?;

    let capture = DxeReadinessCaptureSerDe { hob_list: serializable_hob_list, fv_list: serializable_fv_list };
    serde_json::to_string_pretty(&capture)
        .map_err(|err| format!("Failed to serialize the capture data into JSON: {}", err))
}
