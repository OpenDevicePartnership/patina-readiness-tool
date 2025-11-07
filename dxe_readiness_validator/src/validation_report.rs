//! Structures and methods for collecting and reporting validation results.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use crate::validation_kind::PrettyPrintTable;
use colored::*;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};
use std::collections::BTreeMap;

use crate::validation_kind::ValidationKind;

#[derive(Debug, Default)]
pub struct ValidationReport<'a> {
    // Report is a BTreeMap of Group name and list of violations
    report: BTreeMap<String, Vec<ValidationKind<'a>>>,
}

impl<'a> ValidationReport<'a> {
    pub fn new() -> Self {
        Self { report: BTreeMap::new() }
    }

    pub fn add_violation(&mut self, validation: ValidationKind<'a>) {
        let group_name = validation.name();
        self.report.entry(group_name).or_default().push(validation);
    }

    pub fn append_report(&mut self, mut validation_report: ValidationReport<'a>) {
        self.report.append(&mut validation_report.report);
    }

    pub fn violation_count(&self) -> usize {
        self.report.values().map(Vec::len).sum()
    }

    pub fn show_results(&self) {
        if self.report.is_empty() {
            println!("No violations found.");
        } else {
            self.pretty_print();
        }
    }

    fn pretty_print(&self) {
        println!("{}", "Validation Results:".red().bold());
        for violations in self.report.values() {
            if violations.is_empty() {
                continue;
            }

            println!("──────────────────────────────────────────────────────────────────");
            println!("❌ {}", violations.first().unwrap().header().green().bold());
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(violations.first().unwrap().table_header());

            for (j, violation) in violations.iter().enumerate() {
                table.add_row(violation.table_row((j + 1).to_string()));
            }

            println!("{table}");
            println!("💡 {}", format!("Guidance:\n{}", violations.first().unwrap().guidance()).blue().bold());
        }
    }
}
