//! Dxe Readiness Capture Tool - X64/AArch64 for UefiShell Binary
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
    if #[cfg(all(target_os = "uefi"))] {
        use dxe_readiness_capture::core_start;
        use uefi::prelude::*;
        use uefi::guid;

        #[entry]
        fn main() -> Status {
            uefi::helpers::init().unwrap();

            // Set the logger max level to Info. Anything above this will
            // trigger goblin's debug spew, dumping PE files from memory!
            log::set_max_level(log::LevelFilter::Info);

            // Locate the configuration entry identified by the GUID
            // gEfiHobListGuid
            let hob_list_guid = guid!("7739F24C-93D7-11D4-9A3A-0090273FC14D");
            system::with_config_table(|t| {
                let config_entry = t.iter().find(|ct| ct.guid == hob_list_guid).unwrap();
                let physical_hob_list = config_entry.address;
                core_start(physical_hob_list);
            });

            Status::SUCCESS
        }
    } else {
        fn main() {}
    }
}
