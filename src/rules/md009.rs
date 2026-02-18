//! MD009 - Trailing spaces
//!
//! This rule checks for lines that end with trailing whitespace.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD009;

impl Rule for MD009 {
    fn names(&self) -> &'static [&'static str] {
        &["MD009", "no-trailing-spaces"]
    }

    fn description(&self) -> &'static str {
        "Trailing spaces"
    }

    fn tags(&self) -> &[&'static str] {
        &["whitespace", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md009.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            // Remove line ending to check for trailing spaces
            let trimmed_end = line.trim_end_matches('\n').trim_end_matches('\r');

            // Check if there are trailing spaces (but not if the line is empty)
            if trimmed_end.ends_with(' ') || trimmed_end.ends_with('\t') {
                let trailing_start = trimmed_end.trim_end().len();
                let trailing_count = trimmed_end.len() - trailing_start;

                errors.push(LintError {
                    line_number,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!("Expected: 0; Actual: {}", trailing_count)),
                    error_context: Some(trimmed_end[trailing_start..].to_string()),
                    rule_information: self.information(),
                    error_range: Some((trailing_start + 1, trailing_count)),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(trailing_start + 1),
                        delete_count: Some(trailing_count as i32),
                        insert_text: None,
                    }),
                    suggestion: Some("Remove trailing spaces".to_string()),
                    severity: Severity::Error,
                    fix_only: false,
                });
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
    fn test_md009_no_trailing_spaces() {
        let lines = vec!["# Heading\n", "This is content\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD009.lint(&params).len(), 0);
    }

    #[test]
    fn test_md009_with_trailing_spaces() {
        let lines = vec!["# Heading  \n", "This is content   \n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD009.lint(&params);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 1);
        assert_eq!(errors[1].line_number, 2);
        assert!(errors[0].fix_info.is_some());
    }

    #[test]
    fn test_md009_with_tabs() {
        let lines = vec!["Content\t\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD009.lint(&params).len(), 1);
    }

    #[test]
    fn test_md009_single_trailing_space() {
        let lines = vec!["Hello \n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD009.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().expect("fix_info");
        assert_eq!(fix.edit_column, Some(6));
        assert_eq!(fix.delete_count, Some(1));
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md009_mixed_spaces_and_tabs() {
        let lines = vec!["Text \t \n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD009.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].error_detail.as_deref(),
            Some("Expected: 0; Actual: 3")
        );
    }

    #[test]
    fn test_md009_blank_line_with_spaces() {
        // A line that is only spaces should still trigger
        let lines = vec!["   \n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD009.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_range, Some((1, 3)));
    }

    #[test]
    fn test_md009_no_newline_trailing_spaces() {
        // Last line without newline but with trailing spaces
        let lines = vec!["Content   "];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD009.lint(&params).len(), 1);
    }

    #[test]
    fn test_md009_empty_lines_no_error() {
        let lines = vec!["\n", "\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD009.lint(&params).len(), 0);
    }
}
