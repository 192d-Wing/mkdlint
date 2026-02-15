//! MD022 - Headings should be surrounded by blank lines

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD022;

impl Rule for MD022 {
    fn names(&self) -> &[&'static str] {
        &["MD022", "blanks-around-headings", "blanks-around-headers"]
    }

    fn description(&self) -> &'static str {
        "Headings should be surrounded by blank lines"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers", "blank_lines"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md022.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let headings = params.tokens.filter_by_type("heading");

        for heading in headings {
            let line_num = heading.start_line;

            // Check line before heading
            if line_num > 1 {
                let prev_line = &params.lines[line_num - 2];
                if !prev_line.trim().is_empty() {
                    errors.push(LintError {
                        line_number: line_num,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some("Expected blank line before heading".to_string()),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: Some(FixInfo {
                            line_number: Some(line_num),
                            edit_column: Some(1),
                            delete_count: None,
                            insert_text: Some("\n".to_string()),
                        }),
                        severity: Severity::Error,
                    });
                }
            }

            // Check line after heading
            if line_num < params.lines.len() {
                let next_line = &params.lines[line_num];
                if !next_line.trim().is_empty() {
                    errors.push(LintError {
                        line_number: line_num,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some("Expected blank line after heading".to_string()),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
                        severity: Severity::Error,
                    });
                }
            }
        }

        errors
    }
}
