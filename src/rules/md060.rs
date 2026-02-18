//! MD060 - Dollar signs used before code fence

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD060;

impl Rule for MD060 {
    fn names(&self) -> &'static [&'static str] {
        &["MD060", "dollar-in-code-fence"]
    }

    fn description(&self) -> &'static str {
        "Dollar signs used before commands in fenced code blocks without output"
    }

    fn tags(&self) -> &[&'static str] {
        &["code", "fixable"]
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
                // Calculate the column where the $ appears
                let leading_ws = line.len() - line.trim_start().len();
                let dollar_col = leading_ws + 1; // 1-based column

                // Delete "$ " if there's a space after, otherwise just "$"
                let delete_count = if trimmed.len() > 1 && trimmed.chars().nth(1) == Some(' ') {
                    2 // Delete "$ "
                } else {
                    1 // Delete "$"
                };

                errors.push(LintError {
                    line_number,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: None,
                    error_context: Some(trimmed.to_string()),
                    rule_information: self.information(),
                    error_range: None,
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(dollar_col),
                        delete_count: Some(delete_count),
                        insert_text: None,
                    }),
                    suggestion: Some("Remove the $ prefix from this command".to_string()),
                    severity: Severity::Error,
                    fix_only: false,
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
        lines: &'a [&'a str],
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
        let lines: Vec<&str> = vec!["```bash\n", "echo hello\n", "ls -la\n", "```\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md060_dollar_sign_in_code_block() {
        let rule = MD060;
        let lines: Vec<&str> = vec!["```bash\n", "$ echo hello\n", "$ ls -la\n", "```\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md060_dollar_sign_outside_code_block() {
        let rule = MD060;
        let lines: Vec<&str> = vec!["$ echo hello\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md060_tilde_code_fence() {
        let rule = MD060;
        let lines: Vec<&str> = vec!["~~~\n", "$ npm install\n", "~~~\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md060_mixed_dollar_and_non_dollar() {
        let rule = MD060;
        let lines: Vec<&str> = vec!["```\n", "$ echo hello\n", "hello\n", "```\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md060_fix_dollar_with_space() {
        let rule = MD060;
        let lines: Vec<&str> = vec!["```bash\n", "$ echo hello\n", "```\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());

        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(2)); // Delete "$ "
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md060_fix_dollar_without_space() {
        let rule = MD060;
        let lines: Vec<&str> = vec!["```bash\n", "$echo\n", "```\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());

        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(1)); // Delete "$" only
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md060_fix_indented_dollar() {
        let rule = MD060;
        let lines: Vec<&str> = vec!["```bash\n", "  $ echo hello\n", "```\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());

        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(3)); // Column after 2 spaces
        assert_eq!(fix.delete_count, Some(2)); // Delete "$ "
        assert_eq!(fix.insert_text, None);
    }
}
