//! Heap allocator initialization for DXE readiness capture environment.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use linked_list_allocator::LockedHeap;

#[cfg_attr(not(feature = "uefishell"), global_allocator)]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(free_memory_bottom: usize, free_memory_top: usize) {
    let heap_start = free_memory_bottom as *mut u8;
    let heap_size = free_memory_top - free_memory_bottom;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}
