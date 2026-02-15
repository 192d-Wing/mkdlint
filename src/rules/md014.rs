//! MD014 - Dollar signs used before commands without showing output
//!
//! This rule checks for code blocks that show commands with $ prefix

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD014;

impl Rule for MD014 {
    fn names(&self) -> &[&'static str] {
        &["MD014", "commands-show-output"]
    }

    fn description(&self) -> &'static str {
        "Dollar signs used before commands without showing output"
    }

    fn tags(&self) -> &[&'static str] {
        &["code"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md014.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut in_code_block = false;
        let mut code_block_start = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                if in_code_block {
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    code_block_start = line_number;
                }
            } else if in_code_block && trimmed.starts_with('$') {
                errors.push(LintError {
                    line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(trimmed.to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((1, line.len())),
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

    #[test]
    fn test_md014_no_dollar_signs() {
        let lines = vec![
            "```bash\n".to_string(),
            "echo hello\n".to_string(),
            "```\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD014;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md014_with_dollar_signs() {
        let lines = vec![
            "```bash\n".to_string(),
            "$ echo hello\n".to_string(),
            "```\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD014;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
