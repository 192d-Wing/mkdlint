//! KMD006 - IAL syntax must be well-formed
//!
//! In Kramdown, Inline Attribute Lists (IAL) are written as:
//!   `{: #id .class key="value"}`
//!
//! on their own line following a block element. This rule fires when a line
//! starting with `{:` does not match valid IAL syntax, catching common typos.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

/// A line that starts an IAL block
static IAL_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\{:").unwrap());

/// A valid IAL: `{:` followed by zero or more valid attributes, then `}`
///
/// Valid attributes:
/// - `#id`         — ID selector
/// - `.class`      — class selector
/// - `key="value"` — key-value pair with double quotes
/// - `key='value'` — key-value pair with single quotes
/// - `key`         — boolean attribute
static VALID_IAL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^\{:\s*(?:[#.][^\s}\{]+|[A-Za-z_][\w-]*(?:=(?:"[^"]*"|'[^']*'|[\w-]+))?)\s*(?:\s+(?:[#.][^\s}\{]+|[A-Za-z_][\w-]*(?:=(?:"[^"]*"|'[^']*'|[\w-]+))?))*\s*\}\s*$"#,
    )
    .unwrap()
});

/// An empty IAL `{:}` is also valid (no attributes)
static EMPTY_IAL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\{:\s*\}\s*$").unwrap());

pub struct KMD006;

impl Rule for KMD006 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD006", "valid-ial-syntax"]
    }

    fn description(&self) -> &'static str {
        "IAL (Inline Attribute List) syntax must be well-formed"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "ial", "attributes"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn is_enabled_by_default(&self) -> bool {
        false
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let lines = params.lines;
        let mut in_code_block = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r').trim();

            // Track code fences
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Only check lines that look like IALs
            if !IAL_LINE_RE.is_match(trimmed) {
                continue;
            }

            // Check if it's valid
            if !VALID_IAL_RE.is_match(trimmed) && !EMPTY_IAL_RE.is_match(trimmed) {
                errors.push(LintError {
                    line_number: idx + 1,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!(
                        "Malformed IAL syntax: '{trimmed}' \
                         (expected: {{: #id .class key=\"val\"}})"
                    )),
                    severity: Severity::Error,
                    fix_info: Some(FixInfo {
                        line_number: Some(idx + 1),
                        edit_column: Some(1),
                        delete_count: Some(-1), // Delete the malformed IAL line
                        insert_text: None,
                    }),
                    ..Default::default()
                });
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleParams;
    use std::collections::HashMap;

    fn lint(content: &str) -> Vec<LintError> {
        let lines: Vec<&str> = content.split_inclusive('\n').collect();
        let rule = KMD006;
        rule.lint(&RuleParams {
            name: "test.md",
            version: "0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        })
    }

    #[test]
    fn test_kmd006_valid_id() {
        let errors = lint("# H\n\n{: #my-id}\n");
        assert!(errors.is_empty(), "valid IAL with ID should not fire");
    }

    #[test]
    fn test_kmd006_valid_class() {
        let errors = lint("# H\n\n{: .highlight}\n");
        assert!(errors.is_empty(), "valid IAL with class should not fire");
    }

    #[test]
    fn test_kmd006_valid_combined() {
        let errors = lint("# H\n\n{: #intro .section}\n");
        assert!(
            errors.is_empty(),
            "valid IAL with id and class should not fire"
        );
    }

    #[test]
    fn test_kmd006_valid_key_value() {
        let errors = lint("# H\n\n{: data-x=\"foo\"}\n");
        assert!(
            errors.is_empty(),
            "valid IAL with key=value should not fire"
        );
    }

    #[test]
    fn test_kmd006_malformed_ial() {
        let errors = lint("# H\n\n{: bad!!syntax}\n");
        assert!(
            errors.iter().any(|e| e.rule_names[0] == "KMD006"),
            "should fire on malformed IAL"
        );
    }

    #[test]
    fn test_kmd006_unclosed_ial() {
        let errors = lint("# H\n\n{: #id\n");
        assert!(
            errors.iter().any(|e| e.rule_names[0] == "KMD006"),
            "should fire on unclosed IAL"
        );
    }

    #[test]
    fn test_kmd006_in_code_block_ignored() {
        let errors = lint("# H\n\n```\n{: bad!!stuff}\n```\n");
        assert!(errors.is_empty(), "should not fire inside code blocks");
    }
}
