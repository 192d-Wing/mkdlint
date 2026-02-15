//! MD050 - Strong style should be consistent

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD050;

impl Rule for MD050 {
    fn names(&self) -> &[&'static str] {
        &["MD050", "strong-style"]
    }

    fn description(&self) -> &'static str {
        "Strong style should be consistent"
    }

    fn tags(&self) -> &[&'static str] {
        &["emphasis"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md050.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut double_asterisk_count = 0;
        let mut double_underscore_count = 0;

        for line in params.lines.iter() {
            double_asterisk_count += line.matches("**").count();
            double_underscore_count += line.matches("__").count();
        }

        // If both are used, it's inconsistent
        if double_asterisk_count > 0 && double_underscore_count > 0 {
            errors.push(LintError {
                line_number: 1,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: Some("Mixed strong styles (** and __)".to_string()),
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
    fn test_md050_consistent_double_asterisks() {
        let rule = MD050;
        let lines: Vec<String> = vec!["**bold** text\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md050_mixed_styles() {
        let rule = MD050;
        let lines: Vec<String> = vec!["**bold** and __also bold__\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
