//! Dxe Readiness Validation Tool - X64/AArch64
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: BSD-2-Clause-Patent
//!

use errors::ValidationAppError;
use logger::init_logger;
use std::process;
use validate::ValidationApp;

mod commandline;
mod errors;
mod logger;
mod validate;
mod validation_kind;
mod validation_report;
mod validator;

fn main() {
    // The call to run_main() guarantees that all destructors have finished
    // executing within run_main(), making it safe to call exit().
    let exit_code = run_main();
    process::exit(exit_code);
}

/// Entry point for running the validation application logic.
fn run_main() -> i32 {
    init_logger();

    let mut app = ValidationApp::new();

    if let Err(err) = app.parse_json() {
        eprintln!("{}", err);
        return map_error(&err);
    }

    if let Err(err) = app.validate() {
        eprintln!("{}", err);
        return map_error(&err);
    }

    0 // Success
}

/// Maps a `ValidationAppError` to a platform-level exit code.
///
/// This function is intended to convert high-level application errors into
/// numeric exit codes suitable for CI.
///
/// # Returns
/// - The number of validation errors as `i32` if the error is
///   `ValidationErrors`.
/// - `-1` for all other types of errors, indicating a generic failure.
fn map_error(err: &ValidationAppError) -> i32 {
    match err {
        ValidationAppError::ValidationErrors(count) => *count as i32,
        _ => -1,
    }
}
