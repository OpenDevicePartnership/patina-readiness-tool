//! Dxe Readiness Validation Tool - X64/AArch64
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: BSD-2-Clause-Patent
//!

use clap::Parser;
use commandline::CommandLine;
use logger::init_logger;

mod commandline;
mod logger;
fn main() {
    init_logger();
    log::info!("Hello from Dxe Readiness Validation Tool!\n");
    let args = CommandLine::parse();
    log::info!("Command line arguments: {:#?}", args);
}
