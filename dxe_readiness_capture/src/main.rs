//! Dxe Readiness Capture Tool - X64/AArch64
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: BSD-2-Clause-Patent
//!
// #![cfg(all(target_os = "uefi"))]
#![no_std]
#![no_main]

use stacktrace::StackTrace;
use core::{ffi::c_void, panic::PanicInfo};
use dxe_core::Core; // TODO: Replace this with bump allocator
// use uefi_sdk::{log::Format, serial::{SerialIO, Uart16550}};

mod logger;
use logger::init_logger;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);

    if let Err(err) = unsafe { StackTrace::dump() } {
        log::error!("StackTrace: {}", err);
    }

    loop {}
}

#[cfg_attr(target_os = "uefi", export_name = "efi_main")]
pub extern "efiapi" fn _start(_physical_hob_list: *const c_void) -> ! {
    init_logger();

    log::info!("Hello from Dxe Readiness Capture Tool!\n");
    log::info!("Dead Loop Time\n");
    loop {}
}
