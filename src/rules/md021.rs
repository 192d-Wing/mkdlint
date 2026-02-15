//! MD021 - Multiple spaces inside hashes on closed atx style heading

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD021;

impl Rule for MD021 {
    fn names(&self) -> &[&'static str] {
        &["MD021", "no-multiple-space-closed-atx"]
    }

    fn description(&self) -> &'static str {
        "Multiple spaces inside hashes on closed atx style heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "atx", "atx_closed", "spaces"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md021.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with('#') && trimmed.ends_with('#') {
                let content = trimmed.trim_start_matches('#').trim_end_matches('#');
                if !content.is_empty() {
                    let start_spaces = content.chars().take_while(|&c| c == ' ').count();
                    let end_spaces = content.chars().rev().take_while(|&c| c == ' ').count();

                    if start_spaces > 1 || end_spaces > 1 {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: None,
                            error_context: Some(trimmed.to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: None,
                            fix_info: None,
                            severity: Severity::Error,
                        });
                    }
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

    fn make_params<'a>(lines: &'a [String], tokens: &'a [crate::parser::Token], config: &'a HashMap<String, serde_json::Value>) -> crate::types::RuleParams<'a> {
        crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens,
            config,
        }
    }

    #[test]
    fn test_md021_single_space() {
        let lines: Vec<String> = "# Heading #\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md021_multiple_start_spaces() {
        let lines: Vec<String> = "#  Heading #\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md021_multiple_end_spaces() {
        let lines: Vec<String> = "# Heading  #\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
