//! MD028 - Blank line inside blockquote

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD028;

impl Rule for MD028 {
    fn names(&self) -> &'static [&'static str] {
        &["MD028", "no-blanks-blockquote"]
    }

    fn description(&self) -> &'static str {
        "Blank line inside blockquote"
    }

    fn tags(&self) -> &[&'static str] {
        &["blockquote", "whitespace", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md028.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut in_blockquote = false;
        let mut blank_line = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with('>') {
                if blank_line > 0 && in_blockquote {
                    errors.push(LintError {
                        line_number: blank_line,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: None,
                        error_context: None,
                        rule_information: self.information(),
                        error_range: None,
                        fix_info: Some(FixInfo {
                            line_number: Some(blank_line),
                            edit_column: Some(1),
                            delete_count: Some(-1), // Delete entire line
                            insert_text: None,
                        }),
                        suggestion: Some("Remove blank lines inside blockquote".to_string()),
                        severity: Severity::Error,
                        fix_only: false,
                    });
                }
                in_blockquote = true;
                blank_line = 0;
            } else if trimmed.is_empty() {
                if in_blockquote {
                    blank_line = line_number;
                }
            } else {
                in_blockquote = false;
                blank_line = 0;
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
    fn test_md028_no_blank_in_quote() {
        let lines: Vec<&str> = "> line 1\n> line 2\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let rule = MD028;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md028_blank_in_quote() {
        let lines: Vec<&str> = "> line 1\n\n> line 2\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let rule = MD028;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
    }

    #[test]
    fn test_md028_not_blockquote() {
        let lines: Vec<&str> = "normal text\n\nmore text\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let rule = MD028;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md028_fix_blank_line() {
        let lines: Vec<&str> = "> line 1\n\n> line 2\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let rule = MD028;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.line_number, Some(2));
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(-1)); // Delete entire line
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md028_fix_multiple_blank_lines() {
        let lines: Vec<&str> = vec!["> line 1\n", "\n", "\n", "> line 2\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let rule = MD028;
        let errors = rule.lint(&params);
        // Only the last blank line before the next blockquote is reported
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
    }

    #[test]
    fn test_md028_fix_whitespace_line() {
        let lines: Vec<&str> = vec!["> line 1\n", "   \n", "> line 2\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let rule = MD028;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.delete_count, Some(-1));
    }
}
