//! MD014 - Dollar signs used before commands without showing output
//!
//! This rule checks for code blocks that show commands with $ prefix

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD014;

impl Rule for MD014 {
    fn names(&self) -> &[&'static str] {
        &["MD014", "commands-show-output"]
    }

    fn description(&self) -> &'static str {
        "Dollar signs used before commands without showing output"
    }

    fn tags(&self) -> &[&'static str] {
        &["code", "fixable"]
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
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
            } else if in_code_block && trimmed.starts_with('$') {
                // Calculate position and deletion count
                let leading_ws = line.len() - line.trim_start().len();
                let dollar_pos = leading_ws + 1; // 1-based column

                // Check if there's a space after $
                let delete_count = if trimmed.len() > 1 && trimmed.chars().nth(1) == Some(' ') {
                    2 // Delete "$ "
                } else {
                    1 // Delete "$"
                };

                errors.push(LintError {
                    line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(trimmed.to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((1, line.len())),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(dollar_pos),
                        delete_count: Some(delete_count),
                        insert_text: None,
                    }),
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

    #[test]
    fn test_md014_fix_dollar_with_space() {
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
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(2)); // "$ "
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md014_fix_dollar_without_space() {
        let lines = vec![
            "```bash\n".to_string(),
            "$echo hello\n".to_string(),
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
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(1)); // "$"
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md014_fix_indented_dollar() {
        let lines = vec![
            "```bash\n".to_string(),
            "  $ echo hello\n".to_string(),
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
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(3)); // After "  "
        assert_eq!(fix.delete_count, Some(2)); // "$ "
        assert_eq!(fix.insert_text, None);
    }
}
