//! MD024 - Multiple headings with the same content

use crate::parser::TokenExt;
use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use std::collections::HashSet;

pub struct MD024;

impl Rule for MD024 {
    fn names(&self) -> &[&'static str] {
        &["MD024", "no-duplicate-heading", "no-duplicate-header"]
    }

    fn description(&self) -> &'static str {
        "Multiple headings with the same content"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md024.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut seen_headings = HashSet::new();
        let headings = params.tokens.filter_by_type("heading");

        for heading in headings {
            let text = heading.text.trim();
            let normalized = text.trim_start_matches('#').trim();

            if !normalized.is_empty() && seen_headings.contains(normalized) {
                errors.push(LintError {
                    line_number: heading.start_line,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(normalized.to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: None,
                    fix_info: None,
                    severity: Severity::Error,
                });
            }

            seen_headings.insert(normalized.to_string());
        }

        errors
    }
}
