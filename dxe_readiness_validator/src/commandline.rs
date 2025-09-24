//! Command-line argument parsing for the DXE readiness validator.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use clap::Parser;

#[derive(Default, Parser, Debug)]
pub struct CommandLine {
    #[arg(short, long, help = "File path of the capture.json")]
    pub filename: Option<String>,
}
