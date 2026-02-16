//! MD024 - Multiple headings with the same content

use crate::parser::TokenExt;
use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use std::collections::HashSet;

pub struct MD024;

impl Rule for MD024 {
    fn names(&self) -> &[&'static str] {
        &["MD024", "no-duplicate-heading", "no-duplicate-header"]
    }

    fn description(&self) -> &'static str {
        "Multiple headings with the same content"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md024.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut seen_headings = HashSet::new();
        let headings = params.tokens.filter_by_type("heading");

        for heading in headings {
            let normalized = heading.text.trim();

            if !normalized.is_empty() && seen_headings.contains(normalized) {
                errors.push(LintError {
                    line_number: heading.start_line,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(normalized.to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: None,
                    fix_info: None,
                    suggestion: Some("Use unique content for each heading".to_string()),
                    severity: Severity::Error,
                });
            }

            seen_headings.insert(normalized.to_string());
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
            "# Introduction\n".to_string(),
            "\n".to_string(),
            "## Details\n".to_string(),
            "\n".to_string(),
            "## Conclusion\n".to_string(),
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
        let lines = vec![
            "## Setup\n".to_string(),
            "\n".to_string(),
            "## Usage\n".to_string(),
            "\n".to_string(),
            "## Setup\n".to_string(),
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
        let lines = vec![
            "## FAQ\n".to_string(),
            "\n".to_string(),
            "## FAQ\n".to_string(),
            "\n".to_string(),
            "## FAQ\n".to_string(),
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
        assert_eq!(errors.len(), 2, "Second and third occurrence should error");
    }

    #[test]
    fn test_md024_different_levels_same_text() {
        let tokens = vec![
            make_heading(1, "Overview", 1),
            make_heading(3, "Overview", 2),
        ];
        let lines = vec![
            "# Overview\n".to_string(),
            "\n".to_string(),
            "## Overview\n".to_string(),
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
        assert_eq!(
            errors.len(),
            1,
            "Same text at different levels is still a duplicate"
        );
    }

    #[test]
    fn test_md024_no_fix_info() {
        let tokens = vec![make_heading(1, "Title", 1), make_heading(3, "Title", 2)];
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "## Title\n".to_string(),
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
        assert!(
            errors[0].fix_info.is_none(),
            "MD024 should not have fix_info"
        );
    }
}
