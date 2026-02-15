//! MD034 - Bare URL used

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use once_cell::sync::Lazy;

static URL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s<>]+").unwrap()
});

pub struct MD034;

impl Rule for MD034 {
    fn names(&self) -> &[&'static str] {
        &["MD034", "no-bare-urls"]
    }

    fn description(&self) -> &'static str {
        "Bare URL used"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "url"]
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
                errors.push(LintError {
                    line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(mat.as_str().to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((mat.start() + 1, mat.len())),
                    fix_info: None,
                    severity: Severity::Error,
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
        let lines = vec!["[link](https://example.com)\n".to_string()];

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
        let lines = vec!["Visit https://example.com for more\n".to_string()];

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
}
