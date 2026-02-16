//! MD007 - Unordered list indentation
//!
//! This rule checks that unordered list items have consistent indentation.
//! Each nested level should be indented by a consistent number of spaces
//! (default: 2).

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static UL_MARKER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\s*)[*+\-]\s").unwrap());

pub struct MD007;

impl Rule for MD007 {
    fn names(&self) -> &[&'static str] {
        &["MD007", "ul-indent"]
    }

    fn description(&self) -> &'static str {
        "Unordered list indentation"
    }

    fn tags(&self) -> &[&'static str] {
        &["bullet", "ul", "indentation"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md007.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let indent = params
            .config
            .get("indent")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as usize;

        let mut in_code_block = false;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Track code blocks
            if trimmed.trim_start().starts_with("```") || trimmed.trim_start().starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Check for unordered list markers
            if let Some(caps) = UL_MARKER_RE.captures(trimmed) {
                let leading_spaces = caps.get(1).unwrap().as_str().len();

                // If there's indentation, check it's a multiple of `indent`
                if leading_spaces > 0 && leading_spaces % indent != 0 {
                    let expected = (leading_spaces / indent) * indent;
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!(
                            "Expected: {}; Actual: {}",
                            expected, leading_spaces
                        )),
                        error_context: Some(trimmed.to_string()),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: Some((1, leading_spaces)),
                        fix_info: Some(FixInfo {
                            line_number: None,
                            edit_column: Some(1),
                            delete_count: Some(leading_spaces as i32),
                            insert_text: Some(" ".repeat(expected)),
                        }),
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
    ) -> RuleParams<'a> {
        RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens: &[],
            config,
        }
    }

    #[test]
    fn test_md007_correct_indentation() {
        let lines: Vec<String> = vec![
            "* Item 1\n".to_string(),
            "  * Nested item\n".to_string(),
            "    * Deep nested\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD007;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md007_wrong_indentation() {
        let lines: Vec<String> = vec![
            "* Item 1\n".to_string(),
            "   * Nested item\n".to_string(), // 3 spaces, should be 2 or 4
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD007;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(
            errors[0].error_detail,
            Some("Expected: 2; Actual: 3".to_string())
        );
    }

    #[test]
    fn test_md007_custom_indent() {
        let lines: Vec<String> = vec![
            "* Item 1\n".to_string(),
            "    * Nested item\n".to_string(), // 4 spaces, correct for indent=4
        ];
        let mut config = HashMap::new();
        config.insert("indent".to_string(), serde_json::json!(4));
        let params = make_params(&lines, &config);

        let rule = MD007;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md007_top_level_no_error() {
        let lines: Vec<String> = vec![
            "* Item 1\n".to_string(),
            "* Item 2\n".to_string(),
            "- Item 3\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD007;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md007_in_code_block_ignored() {
        let lines: Vec<String> = vec![
            "```\n".to_string(),
            "   * not a list\n".to_string(),
            "```\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD007;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
