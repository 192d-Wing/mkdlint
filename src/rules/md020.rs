//! MD020 - No space inside hashes on closed atx style heading

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD020;

impl Rule for MD020 {
    fn names(&self) -> &[&'static str] {
        &["MD020", "no-missing-space-closed-atx"]
    }

    fn description(&self) -> &'static str {
        "No space inside hashes on closed atx style heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "atx", "atx_closed", "spaces"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md020.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with('#') && trimmed.ends_with('#') {
                let content = trimmed.trim_start_matches('#').trim_end_matches('#');
                if !content.is_empty() {
                    let has_start_space = content.starts_with(' ');
                    let has_end_space = content.ends_with(' ');

                    if !has_start_space || !has_end_space {
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

    fn make_params<'a>(
        lines: &'a [String],
        tokens: &'a [crate::parser::Token],
        config: &'a HashMap<String, serde_json::Value>,
    ) -> crate::types::RuleParams<'a> {
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
    fn test_md020_valid_closed_atx() {
        let lines: Vec<String> = "# Heading #\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md020_missing_start_space() {
        let lines: Vec<String> = "#Heading #\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md020_not_closed_atx() {
        let lines: Vec<String> = "# Heading\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
