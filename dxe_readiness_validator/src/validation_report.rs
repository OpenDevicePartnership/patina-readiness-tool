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
use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL};
use std::collections::BTreeMap;

use crate::validation_kind::ValidationKind;

#[derive(Debug, Default)]
pub struct ValidationReport<'a> {
    // Report is a BTreeMap of Group name and list of violations
    report: BTreeMap<String, Vec<ValidationKind<'a>>>,
    // Human-readable summaries of the validated data.
    summaries: Vec<String>,
}

impl<'a> ValidationReport<'a> {
    pub fn new() -> Self {
        Self { report: BTreeMap::new(), summaries: Vec::new() }
    }

    pub fn add_summary(&mut self, summary: String) {
        self.summaries.push(summary);
    }

    pub fn add_violation(&mut self, validation: ValidationKind<'a>) {
        let group_name = validation.name();
        self.report.entry(group_name).or_default().push(validation);
    }

    pub fn append_report(&mut self, mut validation_report: ValidationReport<'a>) {
        self.report.append(&mut validation_report.report);
        self.summaries.append(&mut validation_report.summaries);
    }

    pub fn violation_count(&self) -> usize {
        self.report.values().map(Vec::len).sum()
    }

    pub fn show_results(&self) {
        for summary in &self.summaries {
            println!("{}", summary.cyan().bold());
            println!();
        }

        println!();

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
                .load_style(UTF8_FULL)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation_kind::HobValidationKind;
    use patina::pi::serializable::serializable_hob::ResourceDescriptorSerDe;

    fn resource(start: u64, length: u64) -> ResourceDescriptorSerDe {
        ResourceDescriptorSerDe {
            owner: "owner".to_string(),
            resource_type: 0,
            resource_attribute: 0,
            physical_start: start,
            resource_length: length,
        }
    }

    #[test]
    fn test_add_summary_stores_in_order() {
        let mut report = ValidationReport::new();
        report.add_summary("HOB Summary".to_string());
        report.add_summary("FV Summary".to_string());

        assert_eq!(report.summaries, vec!["HOB Summary".to_string(), "FV Summary".to_string()]);
    }

    #[test]
    fn test_append_report_merges_summaries() {
        let mut base = ValidationReport::new();
        base.add_summary("HOB Summary".to_string());

        let mut other = ValidationReport::new();
        other.add_summary("FV Summary".to_string());

        base.append_report(other);

        assert_eq!(base.summaries, vec!["HOB Summary".to_string(), "FV Summary".to_string()]);
    }

    #[test]
    fn test_new_report_has_no_summaries() {
        let report = ValidationReport::new();
        assert!(report.summaries.is_empty());
    }

    #[test]
    fn test_add_violation_and_count() {
        let r1 = resource(0x1000, 0x1000);
        let r2 = resource(0x1800, 0x1000);

        let mut report = ValidationReport::new();
        assert_eq!(report.violation_count(), 0);

        report.add_violation(ValidationKind::Hob(HobValidationKind::OverlappingMemoryRanges { hob1: &r1, hob2: &r2 }));
        report.add_violation(ValidationKind::Hob(HobValidationKind::V1MemoryRangeNotContainedInV2 { hob1: &r1 }));

        assert_eq!(report.violation_count(), 2);
    }

    #[test]
    fn test_append_report_merges_violations() {
        let r1 = resource(0x1000, 0x1000);
        let mut base = ValidationReport::new();

        let mut other = ValidationReport::new();
        other.add_violation(ValidationKind::Hob(HobValidationKind::V1MemoryRangeNotContainedInV2 { hob1: &r1 }));

        base.append_report(other);
        assert_eq!(base.violation_count(), 1);
    }

    #[test]
    fn test_show_results_with_violations() {
        let r1 = resource(0x1000, 0x1000);
        let r2 = resource(0x1800, 0x1000);

        let mut report = ValidationReport::new();
        report.add_summary("HOB Summary".to_string());
        report.add_violation(ValidationKind::Hob(HobValidationKind::OverlappingMemoryRanges { hob1: &r1, hob2: &r2 }));

        // Exercises the pretty_print path.
        report.show_results();
    }

    #[test]
    fn test_show_results_without_violations() {
        let mut report = ValidationReport::new();
        report.add_summary("HOB Summary".to_string());

        // Exercises the "No violations found" path.
        report.show_results();
    }
}
