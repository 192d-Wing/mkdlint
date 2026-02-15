//! MD058 - Tables should be surrounded by blank lines

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD058;

impl Rule for MD058 {
    fn names(&self) -> &[&'static str] {
        &["MD058", "blanks-around-tables"]
    }

    fn description(&self) -> &'static str {
        "Tables should be surrounded by blank lines"
    }

    fn tags(&self) -> &[&'static str] {
        &["table", "blank_lines"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md058.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut table_start = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains('|') && table_start == 0 {
                table_start = line_number;

                // Check for blank line before
                if line_number > 1 {
                    let prev_line = &params.lines[line_number - 2];
                    if !prev_line.trim().is_empty() {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some("Expected blank line before table".to_string()),
                            error_context: None,
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: None,
                            fix_info: None,
                            severity: Severity::Error,
                        });
                    }
                }
            } else if !trimmed.contains('|') && table_start > 0 {
                // End of table
                if !trimmed.is_empty() {
                    errors.push(LintError {
                        line_number: table_start,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some("Expected blank line after table".to_string()),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
                        severity: Severity::Error,
                    });
                }
                table_start = 0;
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
    fn test_md058_table_with_blank_lines() {
        let rule = MD058;
        let lines: Vec<String> = vec![
            "Some text\n".to_string(),
            "\n".to_string(),
            "| Header 1 | Header 2 |\n".to_string(),
            "| -------- | -------- |\n".to_string(),
            "| Cell 1   | Cell 2   |\n".to_string(),
            "\n".to_string(),
            "More text\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md058_table_without_blank_line_before() {
        let rule = MD058;
        let lines: Vec<String> = vec![
            "Some text\n".to_string(),
            "| Header 1 | Header 2 |\n".to_string(),
            "| -------- | -------- |\n".to_string(),
            "| Cell 1   | Cell 2   |\n".to_string(),
            "\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_detail, Some("Expected blank line before table".to_string()));
    }

    #[test]
    fn test_md058_table_without_blank_line_after() {
        let rule = MD058;
        let lines: Vec<String> = vec![
            "\n".to_string(),
            "| Header 1 | Header 2 |\n".to_string(),
            "| -------- | -------- |\n".to_string(),
            "| Cell 1   | Cell 2   |\n".to_string(),
            "More text\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_detail, Some("Expected blank line after table".to_string()));
    }

    #[test]
    fn test_md058_table_at_start_of_file() {
        let rule = MD058;
        let lines: Vec<String> = vec![
            "| Header 1 | Header 2 |\n".to_string(),
            "| -------- | -------- |\n".to_string(),
            "\n".to_string(),
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
