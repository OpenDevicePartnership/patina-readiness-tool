//! Functions for capturing and serializing Firmware Volumes from the system memory map.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use alloc::vec::Vec;
use common::serializable_fv::FirmwareVolumeSerDe;
use patina::pi::{fw_fs::FirmwareVolume, hob::Hob};

use crate::{CaptureResult, capture::CaptureApp};

impl CaptureApp<'_> {
    pub(crate) fn capture_fv(&self) -> CaptureResult<Vec<FirmwareVolumeSerDe>> {
        let fv_list: Vec<FirmwareVolumeSerDe> = self
            .hob_list
            .iter()
            .filter_map(|hob| {
                if let &Hob::FirmwareVolume(&fv) = hob {
                    let mut fv_serde = FirmwareVolumeSerDe::from(
                        unsafe { FirmwareVolume::new_from_address(fv.base_address) }.unwrap(),
                    );
                    fv_serde.fv_base_address = fv.base_address;
                    Some(fv_serde)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(fv_list)
    }
}
