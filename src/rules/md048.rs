//! MD048 - Code fence style

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD048;

impl Rule for MD048 {
    fn names(&self) -> &[&'static str] {
        &["MD048", "code-fence-style"]
    }

    fn description(&self) -> &'static str {
        "Code fence style"
    }

    fn tags(&self) -> &[&'static str] {
        &["code"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md048.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut backtick_count = 0;
        let mut tilde_count = 0;
        let mut first_style_line = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                if backtick_count == 0 && tilde_count == 0 {
                    first_style_line = line_number;
                }
                backtick_count += 1;
            } else if trimmed.starts_with("~~~") {
                if backtick_count == 0 && tilde_count == 0 {
                    first_style_line = line_number;
                }
                tilde_count += 1;
            }
        }

        // If both styles are used, report error
        if backtick_count > 0 && tilde_count > 0 {
            errors.push(LintError {
                line_number: first_style_line,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: Some("Mixed fence styles (``` and ~~~)".to_string()),
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
    fn test_md048_consistent_backticks() {
        let rule = MD048;
        let lines: Vec<String> = vec![
            "```\n".to_string(),
            "code block 1\n".to_string(),
            "```\n".to_string(),
            "\n".to_string(),
            "```\n".to_string(),
            "code block 2\n".to_string(),
            "```\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md048_consistent_tildes() {
        let rule = MD048;
        let lines: Vec<String> = vec![
            "~~~\n".to_string(),
            "code block 1\n".to_string(),
            "~~~\n".to_string(),
            "\n".to_string(),
            "~~~\n".to_string(),
            "code block 2\n".to_string(),
            "~~~\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md048_mixed_styles() {
        let rule = MD048;
        let lines: Vec<String> = vec![
            "```\n".to_string(),
            "code block 1\n".to_string(),
            "```\n".to_string(),
            "\n".to_string(),
            "~~~\n".to_string(),
            "code block 2\n".to_string(),
            "~~~\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
