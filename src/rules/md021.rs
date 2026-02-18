//! MD021 - Multiple spaces inside hashes on closed atx style heading

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD021;

impl Rule for MD021 {
    fn names(&self) -> &'static [&'static str] {
        &["MD021", "no-multiple-space-closed-atx"]
    }

    fn description(&self) -> &'static str {
        "Multiple spaces inside hashes on closed atx style heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "atx", "atx_closed", "spaces", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md021.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with('#') && trimmed.ends_with('#') {
                let content = trimmed.trim_start_matches('#').trim_end_matches('#');
                if !content.is_empty() {
                    let start_spaces = content.chars().take_while(|&c| c == ' ').count();
                    let end_spaces = content.chars().rev().take_while(|&c| c == ' ').count();

                    // Calculate positions for fix_info
                    let leading_hashes = trimmed.chars().take_while(|&c| c == '#').count();
                    let trailing_hashes = trimmed.chars().rev().take_while(|&c| c == '#').count();
                    let leading_ws = line.len() - line.trim_start().len();

                    if start_spaces > 1 {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!("Expected: 1; Actual: {}", start_spaces)),
                            error_context: Some(trimmed.to_string()),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(leading_ws + leading_hashes + 2), // After first space
                                delete_count: Some((start_spaces - 1) as i32),
                                insert_text: None,
                            }),
                            suggestion: Some("Remove extra spaces after opening #".to_string()),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }

                    if end_spaces > 1 {
                        let content_end = trimmed.len() - trailing_hashes;
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!("Expected: 1; Actual: {}", end_spaces)),
                            error_context: Some(trimmed.to_string()),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(leading_ws + content_end - end_spaces + 2), // After first space
                                delete_count: Some((end_spaces - 1) as i32),
                                insert_text: None,
                            }),
                            suggestion: Some("Remove extra spaces before closing #".to_string()),
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
    fn test_md021_single_space() {
        let lines: Vec<&str> = "# Heading #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md021_multiple_start_spaces() {
        let lines: Vec<&str> = "#  Heading #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md021_multiple_end_spaces() {
        let lines: Vec<&str> = "# Heading  #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md021_fix_multiple_start_spaces() {
        let lines: Vec<&str> = "#  Heading #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(3)); // After first space
        assert_eq!(fix.delete_count, Some(1)); // Delete 1 extra space
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md021_fix_multiple_end_spaces() {
        let lines: Vec<&str> = "# Heading  #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.delete_count, Some(1)); // Delete 1 extra space
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md021_fix_many_spaces() {
        let lines: Vec<&str> = "#     Heading #\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD021;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(3)); // After first space
        assert_eq!(fix.delete_count, Some(4)); // Delete 4 extra spaces
        assert_eq!(fix.insert_text, None);
    }
}
