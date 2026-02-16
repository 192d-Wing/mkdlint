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
        &["headings", "headers", "blank_lines", "fixable"]
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
                        suggestion: Some(
                            "Headings should be surrounded by blank lines".to_string(),
                        ),
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
                        fix_info: Some(FixInfo {
                            line_number: Some(line_num + 1),
                            edit_column: Some(1),
                            delete_count: None,
                            insert_text: Some("\n".to_string()),
                        }),
                        suggestion: Some(
                            "Headings should be surrounded by blank lines".to_string(),
                        ),
                        severity: Severity::Error,
                    });
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

    fn make_heading(line: usize, level: u8) -> Token {
        let mut t = Token::new("heading");
        t.start_line = line;
        t.end_line = line;
        t.text = format!("Heading {}", level);
        t.metadata.insert("level".to_string(), level.to_string());
        t
    }

    #[test]
    fn test_md022_no_error_with_blank_lines() {
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "Some text\n".to_string(),
            "\n".to_string(),
            "## Section\n".to_string(),
            "\n".to_string(),
            "More text\n".to_string(),
        ];
        let tokens = vec![make_heading(1, 1), make_heading(5, 2)];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD022.lint(&params);
        assert_eq!(
            errors.len(),
            0,
            "Properly spaced headings should have no errors"
        );
    }

    #[test]
    fn test_md022_missing_blank_before_heading() {
        let lines = vec![
            "# Title\n".to_string(),
            "Some text\n".to_string(),
            "## Section\n".to_string(),
        ];
        let tokens = vec![make_heading(1, 1), make_heading(3, 2)];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD022.lint(&params);
        let before_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_detail.as_deref() == Some("Expected blank line before heading"))
            .collect();
        assert_eq!(before_errors.len(), 1);
        assert_eq!(before_errors[0].line_number, 3);
    }

    #[test]
    fn test_md022_missing_blank_after_heading() {
        let lines = vec!["# Title\n".to_string(), "Some text\n".to_string()];
        let tokens = vec![make_heading(1, 1)];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD022.lint(&params);
        let after_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_detail.as_deref() == Some("Expected blank line after heading"))
            .collect();
        assert_eq!(after_errors.len(), 1);
        assert_eq!(after_errors[0].line_number, 1);
    }

    #[test]
    fn test_md022_fix_info_inserts_blank_before() {
        let lines = vec![
            "# Title\n".to_string(),
            "Some text\n".to_string(),
            "## Section\n".to_string(),
        ];
        let tokens = vec![make_heading(1, 1), make_heading(3, 2)];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD022.lint(&params);
        let before_error = errors
            .iter()
            .find(|e| e.error_detail.as_deref() == Some("Expected blank line before heading"))
            .expect("Should have a before-heading error");

        let fix = before_error
            .fix_info
            .as_ref()
            .expect("Should have fix_info");
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.insert_text, Some("\n".to_string()));
    }

    #[test]
    fn test_md022_heading_at_start_of_file() {
        // First heading at line 1 should not complain about missing blank before
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "Content\n".to_string(),
        ];
        let tokens = vec![make_heading(1, 1)];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD022.lint(&params);
        assert_eq!(
            errors.len(),
            0,
            "Heading at start of file with blank after should be fine"
        );
    }

    #[test]
    fn test_md022_fix_info_inserts_blank_after() {
        let lines = vec!["# Title\n".to_string(), "Some text\n".to_string()];
        let tokens = vec![make_heading(1, 1)];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD022.lint(&params);
        let after_error = errors
            .iter()
            .find(|e| e.error_detail.as_deref() == Some("Expected blank line after heading"))
            .expect("Should have an after-heading error");

        let fix = after_error.fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix.line_number, Some(2));
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.insert_text, Some("\n".to_string()));
    }
}
