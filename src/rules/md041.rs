//! MD041 - First line in a file should be a top-level heading
//!
//! This rule checks that the first line of the file is a top-level (h1) heading.

use crate::parser::TokenExt;
use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD041;

impl Rule for MD041 {
    fn names(&self) -> &[&'static str] {
        &["MD041", "first-line-heading", "first-line-h1"]
    }

    fn description(&self) -> &'static str {
        "First line in a file should be a top-level heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md041.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Skip empty files
        if params.lines.is_empty() {
            return errors;
        }

        // Skip front matter
        let first_content_line = if !params.front_matter_lines.is_empty() {
            params.front_matter_lines.len() + 1
        } else {
            1
        };

        // Find the first heading
        let headings = params.tokens.filter_by_type("heading");

        if let Some(first_heading) = headings.first() {
            // Check if first heading is on the first content line
            if first_heading.start_line != first_content_line {
                errors.push(LintError {
                    line_number: 1,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: None,
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: None,
                    fix_info: None,
                    severity: Severity::Error,
                });
            }
        } else {
            // No heading found
            errors.push(LintError {
                line_number: 1,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: None,
                error_context: None,
                rule_information: self.information().map(|s| s.to_string()),
                error_range: None,
                fix_info: None,
                severity: Severity::Error,
            });
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Token;
    use std::collections::HashMap;

    #[test]
    fn test_md041_starts_with_heading() {
        let tokens = vec![Token {
            token_type: "heading".to_string(),
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 10,
            text: "# Heading".to_string(),
            children: vec![],
            parent: None,
        }];

        let lines = vec!["# Heading\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD041;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md041_no_heading() {
        let tokens = vec![];
        let lines = vec!["Just some text\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD041;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md041_heading_not_first() {
        let tokens = vec![Token {
            token_type: "heading".to_string(),
            start_line: 3,
            start_column: 1,
            end_line: 3,
            end_column: 10,
            text: "# Heading".to_string(),
            children: vec![],
            parent: None,
        }];

        let lines = vec![
            "Some text\n".to_string(),
            "\n".to_string(),
            "# Heading\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD041;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
