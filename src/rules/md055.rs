//! MD055 - Table pipe style

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD055;

impl Rule for MD055 {
    fn names(&self) -> &[&'static str] {
        &["MD055", "table-pipe-style"]
    }

    fn description(&self) -> &'static str {
        "Table pipe style"
    }

    fn tags(&self) -> &[&'static str] {
        &["table"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md055.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            // Check for inconsistent table pipe usage
            if trimmed.contains('|') && trimmed.matches('|').count() > 1 {
                let starts_with_pipe = trimmed.starts_with('|');
                let ends_with_pipe = trimmed.trim_end().ends_with('|');

                if starts_with_pipe != ends_with_pipe {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some("Inconsistent pipe style".to_string()),
                        error_context: Some(trimmed.to_string()),
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
    fn test_md055_consistent_leading_and_trailing_pipes() {
        let rule = MD055;
        let lines: Vec<String> = vec![
            "| Header 1 | Header 2 |\n".to_string(),
            "| -------- | -------- |\n".to_string(),
            "| Cell 1   | Cell 2   |\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md055_no_leading_or_trailing_pipes() {
        let rule = MD055;
        let lines: Vec<String> = vec![
            "Header 1 | Header 2\n".to_string(),
            "-------- | --------\n".to_string(),
            "Cell 1   | Cell 2\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md055_inconsistent_pipe_style() {
        let rule = MD055;
        let lines: Vec<String> = vec!["| Header 1 | Header 2\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md055_trailing_only_pipe() {
        let rule = MD055;
        let lines: Vec<String> = vec!["Header 1 | Header 2 |\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
