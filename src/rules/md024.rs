//! MD024 - Multiple headings with the same content

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD024;

impl Rule for MD024 {
    fn names(&self) -> &'static [&'static str] {
        &["MD024", "no-duplicate-heading", "no-duplicate-header"]
    }

    fn description(&self) -> &'static str {
        "Multiple headings with the same content"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md024.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut heading_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let headings = params.tokens.filter_by_type("heading");

        for heading in headings {
            let normalized = heading.text.trim();

            if !normalized.is_empty() {
                let count = heading_counts.entry(normalized.to_string()).or_insert(0);
                *count += 1;

                // If this is a duplicate (count > 1), report error with fix
                if *count > 1 {
                    let line_number = heading.start_line;
                    let line = &params.lines[line_number - 1];

                    // Find the heading text in the line
                    let heading_start = line.find(normalized);
                    if let Some(start_pos) = heading_start {
                        // Calculate fix: append " (N)" to the heading
                        let new_text = format!("{} ({})", normalized, count);
                        let edit_column = start_pos + normalized.len() + 1; // 1-based

                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!(
                                "Duplicate heading: '{}' (occurrence #{})",
                                normalized, count
                            )),
                            error_context: Some(normalized.to_string()),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(edit_column),
                                delete_count: None,
                                insert_text: Some(format!(" ({})", count)),
                            }),
                            suggestion: Some(format!(
                                "Disambiguate by appending a number: '{}'",
                                new_text
                            )),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Token;
    use std::collections::HashMap;

    fn make_heading(line: usize, text: &str, level: u8) -> Token {
        let mut t = Token::new("heading");
        t.start_line = line;
        t.end_line = line;
        t.text = text.to_string();
        t.metadata.insert("level".to_string(), level.to_string());
        t
    }

    #[test]
    fn test_md024_no_duplicates() {
        let tokens = vec![
            make_heading(1, "Introduction", 1),
            make_heading(3, "Details", 2),
            make_heading(5, "Conclusion", 2),
        ];
        let lines = vec![
            "# Introduction\n",
            "\n",
            "## Details\n",
            "\n",
            "## Conclusion\n",
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD024.lint(&params);
        assert_eq!(errors.len(), 0, "Unique headings should have no errors");
    }

    #[test]
    fn test_md024_duplicate_headings() {
        let tokens = vec![
            make_heading(1, "Setup", 2),
            make_heading(3, "Usage", 2),
            make_heading(5, "Setup", 2),
        ];
        let lines = vec!["## Setup\n", "\n", "## Usage\n", "\n", "## Setup\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD024.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 5);
        assert_eq!(errors[0].error_context, Some("Setup".to_string()));
    }

    #[test]
    fn test_md024_multiple_duplicates() {
        let tokens = vec![
            make_heading(1, "FAQ", 2),
            make_heading(3, "FAQ", 2),
            make_heading(5, "FAQ", 2),
        ];
        let lines = vec!["## FAQ\n", "\n", "## FAQ\n", "\n", "## FAQ\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD024.lint(&params);
        assert_eq!(errors.len(), 2, "Second and third occurrence should error");
    }

    #[test]
    fn test_md024_different_levels_same_text() {
        let tokens = vec![
            make_heading(1, "Overview", 1),
            make_heading(3, "Overview", 2),
        ];
        let lines = vec!["# Overview\n", "\n", "## Overview\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD024.lint(&params);
        assert_eq!(
            errors.len(),
            1,
            "Same text at different levels is still a duplicate"
        );
    }

    #[test]
    fn test_md024_fix_info() {
        let tokens = vec![make_heading(1, "Title", 1), make_heading(3, "Title", 2)];
        let lines = vec!["# Title\n", "\n", "## Title\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD024.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.insert_text, Some(" (2)".to_string()));
        assert_eq!(fix.delete_count, None);
    }

    #[test]
    fn test_md024_fix_multiple_duplicates() {
        let tokens = vec![
            make_heading(1, "FAQ", 2),
            make_heading(3, "FAQ", 2),
            make_heading(5, "FAQ", 2),
        ];
        let lines = vec!["## FAQ\n", "\n", "## FAQ\n", "\n", "## FAQ\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD024.lint(&params);
        assert_eq!(errors.len(), 2);
        // Second occurrence
        assert_eq!(
            errors[0].fix_info.as_ref().unwrap().insert_text,
            Some(" (2)".to_string())
        );
        // Third occurrence
        assert_eq!(
            errors[1].fix_info.as_ref().unwrap().insert_text,
            Some(" (3)".to_string())
        );
    }

    #[test]
    fn test_md024_fix_column_calculation() {
        let tokens = vec![make_heading(1, "Setup", 2), make_heading(3, "Setup", 2)];
        let lines = vec!["## Setup\n", "\n", "## Setup\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD024.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        // "## Setup" -> position after "Setup" is column 9 (1-based)
        assert_eq!(fix.edit_column, Some(9));
    }
}
