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

use core::{ffi::c_void, panic::PanicInfo};
use stacktrace::StackTrace;
// use uefi_sdk::{log::Format, serial::{SerialIO, Uart16550}};

mod bump_allocator;
mod logger;
mod utils;

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
