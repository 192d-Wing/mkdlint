//! MD045 - Images should have alternate text (alt text)

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static IMAGE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"!\[([^\]]*)\]\([^)]+\)").unwrap());

pub struct MD045;

impl Rule for MD045 {
    fn names(&self) -> &[&'static str] {
        &["MD045", "no-alt-text"]
    }

    fn description(&self) -> &'static str {
        "Images should have alternate text (alt text)"
    }

    fn tags(&self) -> &[&'static str] {
        &["accessibility", "images", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md045.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            for cap in IMAGE_RE.captures_iter(line) {
                let alt_text = &cap[1];
                if alt_text.trim().is_empty() {
                    // Calculate column position for the alt text
                    let full_match = cap.get(0).unwrap();
                    let alt_match = cap.get(1).unwrap();
                    let alt_col = alt_match.start() + 1; // 1-based column

                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: None,
                        error_context: Some(full_match.as_str().to_string()),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: Some((full_match.start() + 1, full_match.len())),
                        fix_info: Some(FixInfo {
                            line_number: None,
                            edit_column: Some(alt_col),
                            delete_count: Some(alt_text.len() as i32),
                            insert_text: Some("image".to_string()),
                        }),
                        suggestion: Some(
                            "Add descriptive alt text, e.g., ![description](image.png)".to_string(),
                        ),
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
    fn test_md045_with_alt_text() {
        let rule = MD045;
        let lines: Vec<String> = vec!["![alt text](image.png)\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md045_no_alt_text() {
        let rule = MD045;
        let lines: Vec<String> = vec!["![](image.png)\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md045_whitespace_only_alt() {
        let rule = MD045;
        let lines: Vec<String> = vec!["![  ](image.png)\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
