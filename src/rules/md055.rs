//! MD055 - Table pipe style

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD055;

impl Rule for MD055 {
    fn names(&self) -> &'static [&'static str] {
        &["MD055", "table-pipe-style"]
    }

    fn description(&self) -> &'static str {
        "Table pipe style"
    }

    fn tags(&self) -> &[&'static str] {
        &["table", "fixable"]
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
                    // Calculate leading whitespace
                    let leading_ws = line.len() - line.trim_start().len();

                    // Calculate trailing whitespace (including newline)
                    let line_without_newline = line.trim_end_matches('\n');
                    let trailing_ws =
                        line_without_newline.len() - line_without_newline.trim_end().len();

                    // Generate fix to normalize to both pipes present
                    let fix_info = if starts_with_pipe && !ends_with_pipe {
                        // Add trailing pipe: insert " |" after the last non-whitespace character
                        // Column is 1-based, so we need line_without_newline.len() - trailing_ws
                        // But we want to insert AFTER the last char, so we don't add 1
                        let insert_col = line_without_newline.len() - trailing_ws + 1;
                        Some(FixInfo {
                            line_number: None,
                            edit_column: Some(insert_col),
                            delete_count: None,
                            insert_text: Some(" |".to_string()),
                        })
                    } else if !starts_with_pipe && ends_with_pipe {
                        // Add leading pipe: insert "| " at the start (after leading whitespace)
                        Some(FixInfo {
                            line_number: None,
                            edit_column: Some(leading_ws + 1),
                            delete_count: None,
                            insert_text: Some("| ".to_string()),
                        })
                    } else {
                        None
                    };

                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some("Inconsistent pipe style".to_string()),
                        error_context: Some(trimmed.to_string()),
                        rule_information: self.information(),
                        error_range: None,
                        fix_info,
                        suggestion: Some(
                            "Table rows should have pipes at both the beginning and end, or neither"
                                .to_string(),
                        ),
                        severity: Severity::Error,
                        fix_only: false,
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
    fn test_md055_consistent_leading_and_trailing_pipes() {
        let rule = MD055;
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
    fn test_md055_no_leading_or_trailing_pipes() {
        let rule = MD055;
        let lines: Vec<&str> = vec![
            "Header 1 | Header 2\n",
            "-------- | --------\n",
            "Cell 1   | Cell 2\n",
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
        let lines: Vec<&str> = vec!["| Header 1 | Header 2\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md055_trailing_only_pipe() {
        let rule = MD055;
        let lines: Vec<&str> = vec!["Header 1 | Header 2 |\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md055_fix_missing_trailing_pipe() {
        let rule = MD055;
        let lines: Vec<&str> = vec!["| Header 1 | Header 2\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());

        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(22)); // After "| Header 1 | Header 2" (21 chars + 1)
        assert_eq!(fix.delete_count, None);
        assert_eq!(fix.insert_text, Some(" |".to_string()));
    }

    #[test]
    fn test_md055_fix_missing_leading_pipe() {
        let rule = MD055;
        let lines: Vec<&str> = vec!["Header 1 | Header 2 |\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());

        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(1)); // At start
        assert_eq!(fix.delete_count, None);
        assert_eq!(fix.insert_text, Some("| ".to_string()));
    }

    #[test]
    fn test_md055_fix_indented_table() {
        let rule = MD055;
        let lines: Vec<&str> = vec!["  | Header 1 | Header 2\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());

        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(24)); // After "  | Header 1 | Header 2" (23 chars + 1)
        assert_eq!(fix.insert_text, Some(" |".to_string()));
    }
}
