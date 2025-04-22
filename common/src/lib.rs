#![cfg_attr(not(test), no_std)]

use r_efi::efi::Guid;

#[macro_use]
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serializable_fv::FirmwareVolumeSerDe;
use serializable_hob::HobSerDe;

pub mod serializable_fv;
pub mod serializable_hob;

pub fn format_guid(guid: Guid) -> String {
    // We need this because refi::Guid has private fields
    // and we can't make it derive Serialize (can't modify efi::Guid directly)
    let (time_low, time_mid, time_hi_and_version, clk_seq_hi_res, clk_seq_low, node) = guid.as_fields();
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        time_low,
        time_mid,
        time_hi_and_version,
        clk_seq_hi_res,
        clk_seq_low,
        node[0],
        node[1],
        node[2],
        node[3],
        node[4],
        node[5]
    )
}

pub trait Interval: Clone + Ord {
    fn start(&self) -> u64;
    fn end(&self) -> u64;
    fn length(&self) -> u64 {
        self.end() - self.start()
    }
    fn contains(&self, other: &Self) -> bool {
        self.start() <= other.start() && self.end() >= other.end()
    }
    fn overlaps(&self, other: &Self) -> bool;
    fn merge(&self, other: &Self) -> Self;
    fn try_merge(&self, other: &Self) -> Option<Self> {
        if self.overlaps(other) {
            Some(self.merge(other))
        } else {
            None
        }
    }
    fn merge_intervals(intervals: &[&Self]) -> Vec<Self> {
        if intervals.is_empty() {
            return Vec::new();
        }

        let mut sorted = intervals.to_vec();
        sorted.sort_by(|a, b| a.cmp(b));

        let mut result = vec![sorted[0].clone()];
        for current in sorted.into_iter().skip(1) {
            let last = result.last_mut().unwrap();
            if let Some(merged) = last.try_merge(current) {
                *last = merged;
            } else {
                result.push((*current).clone());
            }
        }

        result
    }
}

/// This structure respresents the actual capture data that will be serialized
/// to JSON.
#[derive(Serialize, Deserialize, Debug)]
pub struct DxeReadinessCaptureSerDe {
    pub hob_list: Vec<HobSerDe>,
    pub fv_list: Vec<FirmwareVolumeSerDe>,
}
