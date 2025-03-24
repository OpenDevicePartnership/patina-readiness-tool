use core::{ffi::c_void, mem};

use mu_pi::hob::{header, PhaseHandoffInformationTable, HANDOFF};

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
