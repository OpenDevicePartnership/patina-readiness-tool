use core::{ffi::c_void, mem, str};

use common::serializable_hob::DeserializableHobList;
use mu_pi::hob::{header, HobList, PhaseHandoffInformationTable, HANDOFF};

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use serde_json_core::to_slice;

pub fn read_phit_hob(physical_hob_list: *const c_void) -> Option<(usize, usize)> {
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

pub const NOT_NULL: &str = "Ptr should not be NULL";

pub fn assert_hob_size<T>(hob: &header::Hob) {
    let hob_len = hob.length as usize;
    let hob_size = mem::size_of::<T>();
    assert_eq!(hob_len, hob_size, "Trying to cast hob of length {hob_len} into a pointer of size {hob_size}");
}

// i wrote this (possibly over-complicated) function because
// in no_std, we don't have access to serde_json::to_string
// we CAN use serde_json_core::to_slice and then convert it to a string,
// but to_slice requires a fix-sized slice, so we use a vec::with capacity and resize it as necessary
// i also choose to use a heap-allocated Vec because i'm concerened about running out of stack space
// (the resulting JSON can be quite large)
// this is not a particularly efficient way to do things but idk how else to do it
pub fn dump_hobs(hob_list: &HobList) -> Option<String> {
    let serializable_list = DeserializableHobList::from(hob_list);

    let mut capacity = 0x1000;
    const MAX_CAPACITY: usize = 0x10000000; // we may need to experiment with these values

    loop {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, 0);

        match to_slice(&serializable_list, &mut buffer[..]) {
            Ok(size) => {
                buffer.truncate(size);
                let s = str::from_utf8(&buffer).expect("Hob list serialization corrupted");
                return Some(String::from(s));
            }
            Err(_) => {
                capacity *= 2;
                if capacity > MAX_CAPACITY {
                    return None;
                }
            }
        }
    }
}
