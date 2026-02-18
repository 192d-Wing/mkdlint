//! MD047 - Files should end with a single newline character

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD047;

impl Rule for MD047 {
    fn names(&self) -> &'static [&'static str] {
        &["MD047", "single-trailing-newline"]
    }

    fn description(&self) -> &'static str {
        "Files should end with a single newline character"
    }

    fn tags(&self) -> &[&'static str] {
        &["blank_lines", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md047.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        if params.lines.is_empty() {
            return errors;
        }

        let last_line = &params.lines[params.lines.len() - 1];

        // Check if file ends with newline
        if !last_line.ends_with('\n') && !last_line.ends_with("\r\n") {
            errors.push(LintError {
                line_number: params.lines.len(),
                rule_names: self.names(),
                rule_description: self.description(),
                error_detail: None,
                error_context: None,
                rule_information: self.information(),
                error_range: None,
                fix_info: Some(FixInfo {
                    line_number: Some(params.lines.len()),
                    edit_column: Some(last_line.len() + 1),
                    delete_count: None,
                    insert_text: Some("\n".to_string()),
                }),
                suggestion: Some("Files should end with a single newline character".to_string()),
                severity: Severity::Error,
                fix_only: false,
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
    fn test_md047_valid_with_newline() {
        let lines = vec!["# Heading\n", "Content\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD047.lint(&params).len(), 0);
    }

    #[test]
    fn test_md047_missing_newline() {
        let lines = vec!["# Heading\n", "Content"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD047.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
        assert!(errors[0].fix_info.is_some());
    }

    #[test]
    fn test_md047_empty_file() {
        let lines: Vec<&str> = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD047.lint(&params).len(), 0);
    }

    #[test]
    fn test_md047_fix_info_appends_newline() {
        let lines = vec!["Content"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD047.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().expect("fix_info");
        assert_eq!(fix.line_number, Some(1));
        assert_eq!(fix.edit_column, Some(8)); // after "Content" (len 7), 1-based
        assert_eq!(fix.insert_text, Some("\n".to_string()));
        assert_eq!(fix.delete_count, None);
    }

    #[test]
    fn test_md047_single_line_with_newline() {
        let lines = vec!["Only line\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD047.lint(&params).len(), 0);
    }

    #[test]
    fn test_md047_single_line_without_newline() {
        let lines = vec!["Only line"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD047.lint(&params).len(), 1);
    }

    #[test]
    fn test_md047_crlf_ending_valid() {
        let lines = vec!["Content\r\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD047.lint(&params).len(), 0);
    }
}
