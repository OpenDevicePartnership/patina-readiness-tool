//! Dxe Readiness Capture Tool - AArch64 QEMU `virt` machine (QemuArmVirtPkg)
//!
//! Targets the QEMU `-machine virt` aarch64 platform, whose PL011 UART is
//! memory-mapped at `0x0900_0000` (matches `PcdSerialRegisterBase` in
//! `QemuArmVirtPkg.dsc`).
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
    if #[cfg(all(target_os = "uefi", target_arch = "aarch64"))] {
        use patina::log::SerialLogger;
        use patina::{log::Format, serial::uart::UartPl011};
        use log::LevelFilter;
        use core::ffi::c_void;
        use dxe_readiness_capture::core_start;

        static LOGGER: SerialLogger<UartPl011> = SerialLogger::new(
            Format::Standard,
            &[],
            log::LevelFilter::Trace,
            UartPl011::new(0x0900_0000),
        );

        fn init_logger() {
            let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
        }

        #[unsafe(export_name = "efi_main")]
        pub extern "efiapi" fn _start(physical_hob_list: *const c_void) -> ! {
            init_logger();
            core_start(physical_hob_list);
            log::info!("Dead Loop");
            loop { core::hint::spin_loop(); }
        }
    } else {
        fn main() {}
    }
}
