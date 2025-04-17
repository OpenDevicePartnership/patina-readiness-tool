//! Dxe Readiness Capture Tool - X64/AArch64
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: BSD-2-Clause-Patent
//!

// no_std and no_main are applicable only when building as an EFI application.
// Tests/other std targets are built as normal Rust binaries, which require main
// and link to std.
#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

// Include all unit testable modules in the crate here.
#[macro_use]
extern crate alloc;
mod capture;
mod capture_fv;
mod capture_hob;
mod hob_util;

cfg_if::cfg_if! {
    // Below code is meant to be compiled as an EFI application. So it should be
    // discarded when the crate is compiling for tests.
    if #[cfg(target_os = "uefi")] {

        mod logger;
        mod allocator;
        use core::{ffi::c_void, panic::PanicInfo};
        use capture::capture;
        use hob_util::read_phit_hob;
        use logger::init_logger;
        use mu_pi::hob::HobList;
        use stacktrace::StackTrace;

        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            log::error!("{}", info);

            if let Err(err) = unsafe { StackTrace::dump() } {
                log::error!("StackTrace: {}", err);
            }

            loop {}
        }

        #[export_name = "efi_main"]
        pub extern "efiapi" fn _start(physical_hob_list: *const c_void) -> ! {
            init_logger();

            log::info!("Hello from Dxe Readiness Capture Tool!");

            let (free_memory_bottom, free_memory_top) = read_phit_hob(physical_hob_list).expect("PHIT HOB was not found.");
            allocator::init(free_memory_bottom, free_memory_top);
            log::info!("Free Memory Bottom: 0x{:X}", free_memory_bottom);
            log::info!("Free Memory Top: 0x{:X}", free_memory_top);

            let mut hob_list = HobList::default();
            hob_list.discover_hobs(physical_hob_list);

            if let Ok(json_str) = capture(&hob_list) {
                log::info!("{}", json_str);
            } else {
                log::info!("Failed to dump HOB list to JSON");
            }

            log::info!("Dead Loop");
            loop {}
        }
    } else {
        fn main() {}
    }
}
