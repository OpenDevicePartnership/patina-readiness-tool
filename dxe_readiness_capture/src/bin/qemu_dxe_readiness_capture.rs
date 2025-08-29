//! Dxe Readiness Capture Tool - X64/Intel QEMU
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
    if #[cfg(all(target_os = "uefi", target_arch = "aarch64"))] {
        use patina_sdk::log::SerialLogger;
        use patina_sdk::{log::Format, serial::uart::UartPl011};
        use log::LevelFilter;
        use core::ffi::c_void;
        use dxe_readiness_capture::core_start;

        static LOGGER: SerialLogger<UartPl011> = SerialLogger::new(
            Format::Standard,
            &[],
            log::LevelFilter::Trace,
            UartPl011::new(0x6000_0000),
        );

        fn init_logger() {
            let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
        }

        #[export_name = "efi_main"]
        pub extern "efiapi" fn _start(physical_hob_list: *const c_void) -> ! {
            init_logger();
            core_start(physical_hob_list);
            log::info!("Dead Loop");
            loop {}
        }
    } else if #[cfg(all(target_os = "uefi", target_arch = "x86_64"))] {
        use patina_sdk::log::SerialLogger;
        use patina_sdk::{log::Format, serial::uart::Uart16550};
        use log::LevelFilter;
        use core::ffi::c_void;
        use dxe_readiness_capture::core_start;

        static LOGGER: SerialLogger<Uart16550> = SerialLogger::new(
            Format::Standard,
            &[],
            log::LevelFilter::Trace,
            Uart16550::Io { base: 0x402 },
        );

        fn init_logger() {
            let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
        }

        #[export_name = "efi_main"]
        pub extern "efiapi" fn _start(physical_hob_list: *const c_void) -> ! {
            init_logger();
            core_start(physical_hob_list);
            log::info!("Dead Loop");
            loop {}
        }
    } else {
        fn main() {}
    }
}
