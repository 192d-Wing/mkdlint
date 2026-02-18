//! MD020 - No space inside hashes on closed atx style heading

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD020;

impl Rule for MD020 {
    fn names(&self) -> &'static [&'static str] {
        &["MD020", "no-missing-space-closed-atx"]
    }

    fn description(&self) -> &'static str {
        "No space inside hashes on closed atx style heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "atx", "atx_closed", "spaces", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md020.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with('#') && trimmed.ends_with('#') {
                let content = trimmed.trim_start_matches('#').trim_end_matches('#');
                if !content.is_empty() {
                    let has_start_space = content.starts_with(' ');
                    let has_end_space = content.ends_with(' ');

                    // Calculate positions for fix_info
                    let leading_hashes = trimmed.chars().take_while(|&c| c == '#').count();
                    let trailing_hashes = trimmed.chars().rev().take_while(|&c| c == '#').count();
                    let leading_ws = line.len() - line.trim_start().len();

                    if !has_start_space {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some("Missing space after opening #".to_string()),
                            error_context: Some(trimmed.to_string()),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(leading_ws + leading_hashes + 1),
                                delete_count: None,
                                insert_text: Some(" ".to_string()),
                            }),
                            suggestion: Some("Add space after opening #".to_string()),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }

                    if !has_end_space {
                        let content_end = trimmed.len() - trailing_hashes;
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some("Missing space before closing #".to_string()),
                            error_context: Some(trimmed.to_string()),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(leading_ws + content_end + 1),
                                delete_count: None,
                                insert_text: Some(" ".to_string()),
                            }),
                            suggestion: Some("Add space before closing #".to_string()),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }
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
    fn test_md020_valid_closed_atx() {
        let lines: Vec<&str> = "# Heading #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md020_missing_start_space() {
        let lines: Vec<&str> = "#Heading #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md020_not_closed_atx() {
        let lines: Vec<&str> = "# Heading\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md020_fix_missing_start_space() {
        let lines: Vec<&str> = "#Heading #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(2)); // After first #
        assert_eq!(fix.delete_count, None);
        assert_eq!(fix.insert_text, Some(" ".to_string()));
    }

    #[test]
    fn test_md020_fix_missing_end_space() {
        let lines: Vec<&str> = "# Heading#\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(10)); // Before closing #
        assert_eq!(fix.delete_count, None);
        assert_eq!(fix.insert_text, Some(" ".to_string()));
    }

    #[test]
    fn test_md020_fix_both_missing() {
        let lines: Vec<&str> = "#Heading#\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD020;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2); // Both start and end errors
        // First error: missing start space
        let fix1 = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix1.edit_column, Some(2));
        assert_eq!(fix1.insert_text, Some(" ".to_string()));
        // Second error: missing end space
        let fix2 = errors[1].fix_info.as_ref().unwrap();
        assert_eq!(fix2.edit_column, Some(9));
        assert_eq!(fix2.insert_text, Some(" ".to_string()));
    }
}
