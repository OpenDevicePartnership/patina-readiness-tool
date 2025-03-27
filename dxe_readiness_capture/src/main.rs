//! Dxe Readiness Capture Tool - X64/AArch64
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: BSD-2-Clause-Patent
//!

// no_std and no_main are applicable only when building as an EFI application.
// Tests are built as normal Rust binaries, which will contain main and link to
// std.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

// Include all unit testable modules in the crate here.
mod bump_allocator;
mod utils;

cfg_if::cfg_if! {
    // Below code is meant to be compiled as an EFI application. So it should be
    // discarded when the crate is compiling for tests.
    if #[cfg(not(test))] {
        use core::{ffi::c_void, panic::PanicInfo};
        use stacktrace::StackTrace;
        mod logger;

        use bump_allocator::ALLOCATOR;
        use logger::init_logger;
        use utils::read_phit_hob;

        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            log::error!("{}", info);

            if let Err(err) = unsafe { StackTrace::dump() } {
                log::error!("StackTrace: {}", err);
            }

            loop {}
        }

        #[cfg_attr(target_os = "uefi", export_name = "efi_main")]
        pub extern "efiapi" fn _start(physical_hob_list: *const c_void) -> ! {
            init_logger();

            let (free_memory_bottom, free_memory_top) = read_phit_hob(physical_hob_list).expect("PHIT HOB was not found.");
            ALLOCATOR.init(free_memory_bottom, free_memory_top);

            log::info!("Hello from Dxe Readiness Capture Tool!\n");
            log::info!("Dead Loop Time\n");
            loop {}
        }
    }
}
