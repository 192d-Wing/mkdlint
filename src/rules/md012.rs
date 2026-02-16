//! MD012 - Multiple consecutive blank lines
//!
//! This rule checks for multiple consecutive blank lines.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD012;

impl Rule for MD012 {
    fn names(&self) -> &[&'static str] {
        &["MD012", "no-multiple-blanks"]
    }

    fn description(&self) -> &'static str {
        "Multiple consecutive blank lines"
    }

    fn tags(&self) -> &[&'static str] {
        &["whitespace", "blank_lines", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md012.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut blank_count = 0;
        let mut first_blank_line = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if blank_count == 0 {
                    first_blank_line = line_number;
                }
                blank_count += 1;
            } else {
                // We hit a non-blank line
                if blank_count > 1 {
                    // Report error on the line after the first blank
                    errors.push(LintError {
                        line_number: first_blank_line + 1,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!("Expected: 1; Actual: {}", blank_count)),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: Some(FixInfo {
                            line_number: Some(first_blank_line + 1),
                            edit_column: Some(1),
                            delete_count: Some(-1), // Delete entire line
                            insert_text: None,
                        }),
                        suggestion: Some("Remove consecutive blank lines".to_string()),
                        severity: Severity::Error,
                    });
                }
                blank_count = 0;
            }
        }

        // Check if file ends with multiple blanks
        if blank_count > 1 {
            errors.push(LintError {
                line_number: first_blank_line + 1,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: Some(format!("Expected: 1; Actual: {}", blank_count)),
                error_context: None,
                rule_information: self.information().map(|s| s.to_string()),
                error_range: None,
                fix_info: Some(FixInfo {
                    line_number: Some(first_blank_line + 1),
                    edit_column: Some(1),
                    delete_count: Some(-1),
                    insert_text: None,
                }),
                suggestion: Some("Remove consecutive blank lines".to_string()),
                severity: Severity::Error,
            });
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_md012_single_blank_lines() {
        let lines = vec![
            "# Heading\n".to_string(),
            "\n".to_string(),
            "Content\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD012;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md012_multiple_blank_lines() {
        let lines = vec![
            "# Heading\n".to_string(),
            "\n".to_string(),
            "\n".to_string(),
            "\n".to_string(),
            "Content\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD012;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3); // Second blank line
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Expected: 1; Actual: 3")
        );
    }

    #[test]
    fn test_md012_no_blank_lines() {
        let lines = vec!["# Heading\n".to_string(), "Content\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD012;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
