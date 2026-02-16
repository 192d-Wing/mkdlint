//! MD047 - Files should end with a single newline character

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD047;

impl Rule for MD047 {
    fn names(&self) -> &[&'static str] {
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
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: None,
                error_context: None,
                rule_information: self.information().map(|s| s.to_string()),
                error_range: None,
                fix_info: Some(FixInfo {
                    line_number: Some(params.lines.len()),
                    edit_column: Some(last_line.len() + 1),
                    delete_count: None,
                    insert_text: Some("\n".to_string()),
                }),
                suggestion: Some("Files should end with a single newline character".to_string()),
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
    fn test_md047_valid_with_newline() {
        let lines = vec!["# Heading\n".to_string(), "Content\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD047;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md047_missing_newline() {
        let lines = vec!["# Heading\n".to_string(), "Content".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD047;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
        assert!(errors[0].fix_info.is_some());
    }

    #[test]
    fn test_md047_empty_file() {
        let lines: Vec<String> = vec![];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD047;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 0);
    }
}
