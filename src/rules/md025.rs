//! MD025 - Multiple top-level headings in the same document

use crate::parser::TokenExt;
use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD025;

impl Rule for MD025 {
    fn names(&self) -> &[&'static str] {
        &["MD025", "single-title", "single-h1"]
    }

    fn description(&self) -> &'static str {
        "Multiple top-level headings in the same document"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md025.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let headings = params.tokens.filter_by_type("heading");
        let mut found_h1 = false;

        for heading in headings {
            // Check if it's an H1 (starts with single #)
            let is_h1 = heading.text.trim_start().starts_with('#')
                && !heading.text.trim_start().starts_with("##");

            if is_h1 {
                if found_h1 {
                    errors.push(LintError {
                        line_number: heading.start_line,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: None,
                        error_context: Some(heading.text.trim().to_string()),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
                        severity: Severity::Error,
                    });
                }
                found_h1 = true;
            }
        }

        errors
    }
}
