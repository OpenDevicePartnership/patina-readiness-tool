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
use validate::validate;

mod commandline;
mod logger;
// mod platform_error;
mod validate;
mod validate_fv;
mod validate_hob;

fn main() -> Result<(), String> {
    init_logger();
    let args = CommandLine::parse();

    let file_path = &args.filename;
    log::info!("File path: {}", file_path);

    validate(file_path)
}
