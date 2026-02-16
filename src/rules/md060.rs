//! MD060 - Dollar signs used before code fence

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD060;

impl Rule for MD060 {
    fn names(&self) -> &[&'static str] {
        &["MD060", "dollar-in-code-fence"]
    }

    fn description(&self) -> &'static str {
        "Dollar signs used before commands in fenced code blocks without output"
    }

    fn tags(&self) -> &[&'static str] {
        &["code"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md060.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut in_code_block = false;
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
            } else if in_code_block && trimmed.starts_with('$') {
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
    fn test_md060_no_dollar_signs() {
        let rule = MD060;
        let lines: Vec<String> = vec![
            "```bash\n".to_string(),
            "echo hello\n".to_string(),
            "ls -la\n".to_string(),
            "```\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md060_dollar_sign_in_code_block() {
        let rule = MD060;
        let lines: Vec<String> = vec![
            "```bash\n".to_string(),
            "$ echo hello\n".to_string(),
            "$ ls -la\n".to_string(),
            "```\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md060_dollar_sign_outside_code_block() {
        let rule = MD060;
        let lines: Vec<String> = vec!["$ echo hello\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md060_tilde_code_fence() {
        let rule = MD060;
        let lines: Vec<String> = vec![
            "~~~\n".to_string(),
            "$ npm install\n".to_string(),
            "~~~\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md060_mixed_dollar_and_non_dollar() {
        let rule = MD060;
        let lines: Vec<String> = vec![
            "```\n".to_string(),
            "$ echo hello\n".to_string(),
            "hello\n".to_string(),
            "```\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
