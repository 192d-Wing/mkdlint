//! MD056 - Table column count

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD056;

impl Rule for MD056 {
    fn names(&self) -> &'static [&'static str] {
        &["MD056", "table-column-count"]
    }

    fn description(&self) -> &'static str {
        "Table column count"
    }

    fn tags(&self) -> &[&'static str] {
        &["table"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md056.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut in_table = false;
        let mut expected_cols = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains('|') {
                let col_count = trimmed.matches('|').count() - 1;

                if !in_table {
                    expected_cols = col_count;
                    in_table = true;
                } else if col_count != expected_cols {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!(
                            "Expected: {} columns; Actual: {} columns",
                            expected_cols, col_count
                        )),
                        error_context: Some(trimmed.to_string()),
                        rule_information: self.information(),
                        error_range: None,
                        fix_info: None,
                        suggestion: Some(
                            "Ensure all table rows have the same number of columns".to_string(),
                        ),
                        severity: Severity::Error,
                        fix_only: false,
                    });
                }
            } else if !trimmed.is_empty() {
                in_table = false;
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
    fn test_md056_consistent_column_count() {
        let rule = MD056;
        let lines: Vec<&str> = vec![
            "| Header 1 | Header 2 |\n",
            "| -------- | -------- |\n",
            "| Cell 1   | Cell 2   |\n",
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md056_inconsistent_column_count() {
        let rule = MD056;
        let lines: Vec<&str> = vec![
            "| Header 1 | Header 2 |\n",
            "| -------- | -------- |\n",
            "| Cell 1   | Cell 2   | Cell 3 |\n",
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md056_single_row_table() {
        let rule = MD056;
        let lines: Vec<&str> = vec!["| Header 1 | Header 2 |\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md056_separate_tables_reset() {
        let rule = MD056;
        let lines: Vec<&str> = vec![
            "| A | B |\n",
            "| - | - |\n",
            "\n",
            "Some text\n",
            "\n",
            "| A | B | C |\n",
            "| - | - | - |\n",
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
