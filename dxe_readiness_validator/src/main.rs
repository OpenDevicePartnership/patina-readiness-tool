//! Dxe Readiness Validation Tool - X64/AArch64
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: BSD-2-Clause-Patent
//!

use logger::init_logger;
use std::process::ExitCode;
use validate::ValidationApp;

mod commandline;
mod logger;
mod validate;

pub type ValidationResult = Result<(), String>;

fn main() -> Result<ExitCode, String> {
    init_logger();
    let mut app = ValidationApp::new();
    let n_errors = app.validate()?;
    Ok(ExitCode::from(n_errors as u8))
}
