//! Dxe Readiness Capture Library - X64/AArch64
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

// no_std and no_main are applicable only when building as an EFI application.
// Tests/other std targets are built as normal Rust binaries, which require main
// and link to std.
#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

cfg_if::cfg_if! {
    // Below code is meant to be compiled as an EFI application. So it should be
    // discarded when the crate is compiling for tests.
    if #[cfg(target_os = "uefi")] {
        #[macro_use]
        extern crate alloc;
        mod allocator;
        mod capture;
        use core::{ffi::c_void, panic::PanicInfo};
        use patina_stacktrace::StackTrace;
        use capture::CaptureApp;
        use alloc::string::String;
        pub type CaptureResult<T> = Result<T, String>;

        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            log::error!("{}", info);

            if let Err(err) = unsafe { StackTrace::dump() } {
                log::error!("StackTrace: {}", err);
            }

            loop {}
        }

        // Called by platform-specific binaries after initializing the logger.
        pub fn core_start(physical_hob_list: *const c_void) {
            log::info!("Dxe Readiness Capture Tool");

            let app = CaptureApp::new(physical_hob_list);

            if let Ok(json_str) = app.capture() {
                log::info!("{}", json_str);
            } else {
                log::info!("Failed to dump HOB list to JSON");
            }
        }
    }
}
