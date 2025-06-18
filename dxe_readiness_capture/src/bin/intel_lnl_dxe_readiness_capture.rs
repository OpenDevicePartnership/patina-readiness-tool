//! Dxe Readiness Capture Tool - X64/Intel Lunar Lake(LNL)
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

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi", target_arch = "x86_64"))] {
        use log::LevelFilter;
        use core::sync::atomic::{AtomicPtr, Ordering};
        use patina_sdk::{log::{Format, SerialLogger}, serial::uart::Uart16550};
        use core::ffi::c_void;
        use dxe_readiness_capture::core_start;

        static mut LOGGER: Option<SerialLogger<Uart16550>> = None;

        const ALTERNATIVE_REGISTER_STRIDE: usize = 1;
        const ASSUMED_REGISTER_STRIDE: usize = 4;
        const ASSUMED_UART_ADDRESS: usize = 0xFE02E000;
        const UART_COMPONENT_REG: usize = 0x3F;
        const UART_COMPONENT_IDENTIFICATION_CODE: u32 = 0x44570110;

        pub(crate) fn get_logger() -> SerialLogger<'static, Uart16550> {
            SerialLogger::new(
                Format::Standard,
                &[],
                log::LevelFilter::Trace,
                Uart16550::Mmio {
                    base: ASSUMED_UART_ADDRESS,
                    reg_stride: get_intel_uart_reg_stride(ASSUMED_UART_ADDRESS),
                },
            )
        }

        fn get_intel_uart_reg_stride(mmio_base: usize) -> usize {
            // Get the component register at the assumed register stride
            let component_register: AtomicPtr<u32> = AtomicPtr::new(
                (mmio_base + (UART_COMPONENT_REG * ASSUMED_REGISTER_STRIDE)) as *mut u32,
            );

            // Read the component register. If the component ID is correct, the assumed register stride
            // must be correct. Otherwise, use the alternative register stride
            match unsafe { core::ptr::read_volatile(component_register.load(Ordering::Relaxed)) } {
                UART_COMPONENT_IDENTIFICATION_CODE => ASSUMED_REGISTER_STRIDE,
                _ => ALTERNATIVE_REGISTER_STRIDE,
            }
        }

        #[allow(static_mut_refs)]
        fn init_logger() {
            let logger_ref: &'static SerialLogger<'static, Uart16550> = unsafe {
                LOGGER = Some(get_logger());
                LOGGER.as_ref().unwrap()
            };
            let _ = log::set_logger(logger_ref).map(|()| log::set_max_level(LevelFilter::Info));
        }

        #[export_name = "efi_main"]
        pub extern "efiapi" fn _start(physical_hob_list: *const c_void) -> ! {
            init_logger();
            core_start(physical_hob_list);
        }
    } else {
        fn main() {}
    }
}
