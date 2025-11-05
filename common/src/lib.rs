//! Common utilities, types, and serialization helpers.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
#![cfg_attr(not(test), no_std)]

use patina::pi::serializable::serializable_hob::HobSerDe;

extern crate alloc;

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serializable_fv::FirmwareVolumeSerDe;

pub mod serializable_fv;

/// This structure respresents the actual capture data that will be serialized
/// to JSON.
#[derive(Serialize, Deserialize, Debug)]
pub struct DxeReadinessCaptureSerDe {
    pub hob_list: Vec<HobSerDe>,
    pub fv_list: Vec<FirmwareVolumeSerDe>,
}
