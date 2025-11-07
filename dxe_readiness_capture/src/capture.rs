//! Capture logic for extracting HOBs and Firmware Volumes from memory.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
// Clippy runs with "--all-targets," which includes "--tests." This module is
// marked to allow dead code to suppress Clippy warnings. Remove this once
// enough tests have been added.
#![allow(dead_code)]
#![allow(unused)]

mod fv;
mod hob;

use common::serializable_fv::FirmwareVolumeSerDe;
use common::DxeReadinessCaptureSerDe;
use core::{ffi::c_void, mem, str};
use mu_pi::serializable::serializable_hob::HobSerDe;

use crate::allocator;
use crate::CaptureResult;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use mu_pi::hob::header::Hob;
use mu_pi::hob::{header, HobList, PhaseHandoffInformationTable, HANDOFF};

pub struct CaptureApp<'a> {
    pub(crate) hob_list: HobList<'a>,
}

impl CaptureApp<'_> {
    pub fn new(physical_hob_list: *const c_void) -> Self {
        let (free_memory_bottom, free_memory_top) =
            Self::read_phit_hob(physical_hob_list).expect("PHIT HOB was not found.");

        if cfg!(not(feature = "uefishell")) {
            allocator::init(free_memory_bottom, free_memory_top);
        }

        log::info!("Free Memory Bottom: 0x{:X}", free_memory_bottom);
        log::info!("Free Memory Top: 0x{:X}", free_memory_top);

        let mut hob_list = HobList::default();
        hob_list.discover_hobs(physical_hob_list);

        Self { hob_list }
    }

    fn read_phit_hob(physical_hob_list: *const c_void) -> Option<(usize, usize)> {
        if physical_hob_list.is_null() {
            panic!("HOB list pointer is null!");
        }

        let hob_header: *const Hob = physical_hob_list as *const Hob;
        const NOT_NULL: &str = "Ptr should not be NULL";

        // The PHIT HOB should always be first
        let current_header = unsafe { hob_header.cast::<Hob>().as_ref().expect(NOT_NULL) };
        if current_header.r#type == HANDOFF {
            Self::assert_hob_size::<PhaseHandoffInformationTable>(current_header);
            let phit_hob = unsafe { hob_header.cast::<PhaseHandoffInformationTable>().as_ref().expect(NOT_NULL) };
            return Some((phit_hob.free_memory_bottom as usize, phit_hob.free_memory_top as usize));
        }

        None
    }

    fn assert_hob_size<T>(hob: &Hob) {
        let hob_len = hob.length as usize;
        let hob_size = mem::size_of::<T>();
        assert_eq!(hob_len, hob_size, "Trying to cast hob of length {hob_len} into a pointer of size {hob_size}");
    }

    pub fn capture(&self) -> CaptureResult<String> {
        let serializable_hob_list: Vec<HobSerDe> = self.capture_hob()?;
        let serializable_fv_list: Vec<FirmwareVolumeSerDe> = self.capture_fv()?;

        let capture = DxeReadinessCaptureSerDe { hob_list: serializable_hob_list, fv_list: serializable_fv_list };
        serde_json::to_string_pretty(&capture)
            .map_err(|err| format!("Failed to serialize the capture data into JSON: {}", err))
    }
}
