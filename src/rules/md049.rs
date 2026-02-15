//! MD049 - Emphasis style should be consistent

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD049;

impl Rule for MD049 {
    fn names(&self) -> &[&'static str] {
        &["MD049", "emphasis-style"]
    }

    fn description(&self) -> &'static str {
        "Emphasis style should be consistent"
    }

    fn tags(&self) -> &[&'static str] {
        &["emphasis"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md049.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut asterisk_count = 0;
        let mut underscore_count = 0;

        for line in params.lines.iter() {
            asterisk_count += line.matches("*").count();
            underscore_count += line.matches("_").count();
        }

        // Simple check - if both are used significantly, it's inconsistent
        if asterisk_count > 2 && underscore_count > 2 {
            errors.push(LintError {
                line_number: 1,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: Some("Mixed emphasis styles (* and _)".to_string()),
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
    fn test_md049_consistent_asterisks() {
        let rule = MD049;
        let lines: Vec<String> = vec!["*one* and *two* and *three*\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md049_mixed_styles() {
        let rule = MD049;
        let lines: Vec<String> = vec!["*one* and *two* and _three_ and _four_ and _five_\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
