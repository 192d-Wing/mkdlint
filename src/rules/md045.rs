//! MD045 - Images should have alternate text (alt text)

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static IMAGE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"!\[([^\]]*)\]\([^)]+\)").expect("valid regex"));

pub struct MD045;

impl Rule for MD045 {
    fn names(&self) -> &'static [&'static str] {
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
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: None,
                        error_context: Some(full_match.as_str().to_string()),
                        rule_information: self.information(),
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

    #[test]
    fn test_md045_with_alt_text() {
        let lines = vec!["![alt text](image.png)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 0);
    }

    #[test]
    fn test_md045_no_alt_text() {
        let lines = vec!["![](image.png)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 1);
    }

    #[test]
    fn test_md045_whitespace_only_alt() {
        let lines = vec!["![  ](image.png)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 1);
    }

    #[test]
    fn test_md045_multiple_images_one_line() {
        let lines = vec!["![](a.png) and ![](b.png)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 2);
    }

    #[test]
    fn test_md045_mixed_valid_and_missing() {
        let lines = vec!["![ok](a.png) ![](b.png)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 1);
    }

    #[test]
    fn test_md045_special_chars_in_alt() {
        let lines = vec!["![diagram: A -> B](flow.png)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 0);
    }

    #[test]
    fn test_md045_fix_info() {
        let lines = vec!["![](photo.jpg)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD045.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().expect("fix_info");
        assert_eq!(fix.edit_column, Some(3));
        assert_eq!(fix.delete_count, Some(0));
        assert_eq!(fix.insert_text, Some("image".to_string()));
    }

    #[test]
    fn test_md045_url_image() {
        let lines = vec!["![](https://example.com/img.png)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 1);
    }

    #[test]
    fn test_md045_regular_link_ignored() {
        let lines = vec!["[text](link.html)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD045.lint(&params).len(), 0);
    }
}
