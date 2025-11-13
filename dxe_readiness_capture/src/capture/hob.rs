//! Functions for capturing and serializing HOBs from the system memory map.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use alloc::vec::Vec;
use patina::pi::serializable::serializable_hob::HobSerDe;

use crate::{CaptureResult, capture::CaptureApp};

impl CaptureApp<'_> {
    pub(crate) fn capture_hob(&self) -> CaptureResult<Vec<HobSerDe>> {
        let fv_list: Vec<HobSerDe> = self.hob_list.iter().map(HobSerDe::from).collect();
        Ok(fv_list)
    }
}
