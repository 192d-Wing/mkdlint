//! MD034 - Bare URL used

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static URL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://[^\s<>]+").expect("valid regex"));

pub struct MD034;

impl Rule for MD034 {
    fn names(&self) -> &'static [&'static str] {
        &["MD034", "no-bare-urls"]
    }

    fn description(&self) -> &'static str {
        "Bare URL used"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "url", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md034.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            // Skip if line contains markdown link syntax
            if line.contains("](") || line.contains("<http") {
                continue;
            }

            for mat in URL_RE.find_iter(line) {
                let url = mat.as_str();
                errors.push(LintError {
                    line_number,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: None,
                    error_context: Some(url.to_string()),
                    rule_information: self.information(),
                    error_range: Some((mat.start() + 1, mat.len())),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(mat.start() + 1),
                        delete_count: Some(mat.len() as i32),
                        insert_text: Some(format!("<{}>", url)),
                    }),
                    suggestion: Some(
                        "Use angle brackets for bare URLs: <http://example.com>".to_string(),
                    ),
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

    #[test]
    fn test_md034_with_markdown_link() {
        let lines = vec!["[link](https://example.com)\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD034;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md034_bare_url() {
        let lines = vec!["Visit https://example.com for more\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD034;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md034_fix_info() {
        let lines = vec!["Visit https://example.com for more\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD034;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        // "Visit https://example.com for more" -> URL starts at column 7 (1-based)
        assert_eq!(fix.edit_column, Some(7));
        // "https://example.com" is 19 chars
        assert_eq!(fix.delete_count, Some(19));
        assert_eq!(fix.insert_text, Some("<https://example.com>".to_string()));
    }

    #[test]
    fn test_md034_fix_info_at_start() {
        let lines = vec!["http://test.org/path\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD034;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(20)); // "http://test.org/path" is 20 chars
        assert_eq!(fix.insert_text, Some("<http://test.org/path>".to_string()));
    }
}
