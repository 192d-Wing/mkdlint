//! MD010 - Hard tabs
//!
//! This rule checks for hard tab characters instead of spaces.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD010;

impl Rule for MD010 {
    fn names(&self) -> &'static [&'static str] {
        &["MD010", "no-hard-tabs"]
    }

    fn description(&self) -> &'static str {
        "Hard tabs"
    }

    fn tags(&self) -> &[&'static str] {
        &["whitespace", "hard_tab", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md010.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            // Find all tab characters in the line
            let mut column = 1;
            for ch in line.chars() {
                if ch == '\t' {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!("Column: {}", column)),
                        error_context: None,
                        rule_information: self.information(),
                        error_range: Some((column, 1)),
                        fix_info: Some(FixInfo {
                            line_number: None,
                            edit_column: Some(column),
                            delete_count: Some(1),
                            insert_text: Some("    ".to_string()), // Replace with 4 spaces
                        }),
                        suggestion: Some("Replace hard tabs with spaces".to_string()),
                        severity: Severity::Error,
                    });
                }
                column += 1;

                // Stop at newline
                if ch == '\n' || ch == '\r' {
                    break;
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_md010_no_tabs() {
        let lines = vec!["# Heading\n", "    Indented with spaces\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD010;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md010_with_tabs() {
        let lines = vec!["\tTabbed line\n", "Normal\tline with tab\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD010;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 1);
        assert_eq!(errors[1].line_number, 2);
        assert!(errors[0].fix_info.is_some());
    }

    #[test]
    fn test_md010_fix_info_correct_column() {
        let lines = vec!["abc\tdef\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };
        let rule = MD010;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(4)); // tab at 4th character (1-based)
        assert_eq!(fix.delete_count, Some(1));
        assert_eq!(fix.insert_text, Some("    ".to_string()));
        // error_range should match
        assert_eq!(errors[0].error_range, Some((4, 1)));
    }

    #[test]
    fn test_md010_multiple_tabs_same_line() {
        let lines = vec!["\t\ttwo tabs\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };
        let rule = MD010;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].error_range, Some((1, 1)));
        assert_eq!(errors[1].error_range, Some((2, 1)));
    }
}
