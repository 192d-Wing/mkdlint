//! MD009 - Trailing spaces
//!
//! This rule checks for lines that end with trailing whitespace.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD009;

impl Rule for MD009 {
    fn names(&self) -> &[&'static str] {
        &["MD009", "no-trailing-spaces"]
    }

    fn description(&self) -> &'static str {
        "Trailing spaces"
    }

    fn tags(&self) -> &[&'static str] {
        &["whitespace"]
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
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: Some(format!("Expected: 0; Actual: {}", trailing_count)),
                    error_context: Some(trimmed_end[trailing_start..].to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((trailing_start + 1, trailing_count)),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(trailing_start + 1),
                        delete_count: Some(trailing_count as i32),
                        insert_text: None,
                    }),
                    severity: Severity::Error,
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
        let lines = vec!["# Heading\n".to_string(), "This is content\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD009;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md009_with_trailing_spaces() {
        let lines = vec![
            "# Heading  \n".to_string(),
            "This is content   \n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD009;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 1);
        assert_eq!(errors[1].line_number, 2);
        assert!(errors[0].fix_info.is_some());
    }

    #[test]
    fn test_md009_with_tabs() {
        let lines = vec!["Content\t\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD009;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
