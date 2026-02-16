//! MD044 - Proper names should have the correct capitalization

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD044;

impl Rule for MD044 {
    fn names(&self) -> &[&'static str] {
        &["MD044", "proper-names"]
    }

    fn description(&self) -> &'static str {
        "Proper names should have the correct capitalization"
    }

    fn tags(&self) -> &[&'static str] {
        &["spelling"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md044.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Read names from config or use defaults
        let names: Vec<String> = params.config
            .get("names")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(|| vec![
                "JavaScript".to_string(),
                "TypeScript".to_string(),
                "GitHub".to_string(),
                "Node.js".to_string(),
            ]);

        let check_code_blocks = params.config
            .get("code_blocks")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Build lookup pairs: (lowercase, correct)
        let proper_names: Vec<(String, String)> = names.iter()
            .map(|name| (name.to_lowercase(), name.clone()))
            .collect();

        let mut in_code_block = false;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip code block content unless configured to check
            if in_code_block && !check_code_blocks {
                continue;
            }

            let lower_line = line.to_lowercase();

            for (incorrect, correct) in &proper_names {
                if lower_line.contains(incorrect.as_str()) && !line.contains(correct.as_str()) {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!("Expected: {}; Actual: {}", correct, incorrect)),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
                        severity: Severity::Error,
                    });
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
        config: &'a HashMap<String, serde_json::Value>,
    ) -> crate::types::RuleParams<'a> {
        crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens: &[],
            config,
        }
    }

    #[test]
    fn test_md044_default_names() {
        let rule = MD044;
        let lines = vec![
            "I love javascript and github.\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md044_correct_names() {
        let rule = MD044;
        let lines = vec![
            "I love JavaScript and GitHub.\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md044_config_names() {
        let rule = MD044;
        let lines = vec![
            "Use rust for everything.\n".to_string(),
        ];
        let mut config = HashMap::new();
        config.insert("names".to_string(), serde_json::json!(["Rust"]));
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md044_code_block_excluded() {
        let rule = MD044;
        let lines = vec![
            "```\n".to_string(),
            "javascript code\n".to_string(),
            "```\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0); // code blocks excluded by default
    }

    #[test]
    fn test_md044_code_block_included() {
        let rule = MD044;
        let lines = vec![
            "```\n".to_string(),
            "javascript code\n".to_string(),
            "```\n".to_string(),
        ];
        let mut config = HashMap::new();
        config.insert("code_blocks".to_string(), serde_json::json!(true));
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1); // code blocks checked when configured
    }
}
