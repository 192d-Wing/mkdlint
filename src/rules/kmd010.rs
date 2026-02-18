//! KMD010 - Inline IAL syntax must be well-formed
//!
//! In Kramdown, IALs can appear inline on span elements:
//!   `*emphasis*{: .class}`, `` `code`{: #id} ``, `[link](url){: key="val"}`
//!
//! KMD006 already validates whole-line IALs (lines starting with `{:`).
//! This rule validates `{:...}` occurrences that appear *within* a line
//! (i.e., inline on spans rather than as standalone block IALs).

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use std::sync::LazyLock;

/// Finds all `{:...}` occurrences within a line (inline IALs)
static INLINE_IAL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{:[^}]*\}").expect("valid regex"));

/// A valid IAL body: zero or more valid attributes
static VALID_IAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^\{:\s*(?:[#.][^\s}\{]+|[A-Za-z_][\w-]*(?:=(?:"[^"]*"|'[^']*'|[\w-]+))?)?\s*(?:\s+(?:[#.][^\s}\{]+|[A-Za-z_][\w-]*(?:=(?:"[^"]*"|'[^']*'|[\w-]+))?))*\s*\}$"#,
    )
    .unwrap()
});

/// An empty IAL `{:}` is also valid
static EMPTY_IAL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\{:\s*\}$").expect("valid regex"));

pub struct KMD010;

impl Rule for KMD010 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD010", "inline-ial-syntax"]
    }

    fn description(&self) -> &'static str {
        "Inline IAL syntax must be well-formed"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "ial", "attributes", "fixable"]
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
            let line_no_newline = line.trim_end_matches('\n').trim_end_matches('\r');
            // leading_offset is the number of bytes stripped by .trim() on the left
            let leading_offset = line_no_newline.len() - line_no_newline.trim_start().len();
            let trimmed = line_no_newline.trim();

            // Track code fences
            if crate::helpers::is_code_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Skip whole-line IALs — those are handled by KMD006
            if trimmed.starts_with("{:") && trimmed.ends_with('}') && !trimmed.contains('\n') {
                // Check if the entire line is just one IAL (block IAL)
                // A block IAL has no other content before the {: on this line
                let before_ial = trimmed.find("{:").map(|p| &trimmed[..p]).unwrap_or("");
                if before_ial.trim().is_empty() {
                    continue;
                }
            }

            // Find all inline {: ...} occurrences
            for mat in INLINE_IAL_RE.find_iter(trimmed) {
                let ial_text = mat.as_str();

                // Skip if this is the whole line trimmed (block IAL, handled by KMD006)
                if ial_text == trimmed {
                    continue;
                }

                if !VALID_IAL_RE.is_match(ial_text) && !EMPTY_IAL_RE.is_match(ial_text) {
                    // Column is 1-based: leading whitespace + match start within trimmed + 1
                    let col = leading_offset + mat.start() + 1;
                    errors.push(LintError {
                        line_number: idx + 1,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!(
                            "Malformed inline IAL syntax: '{ial_text}' \
                             (expected: {{: #id .class key=\"val\"}})"
                        )),
                        severity: Severity::Error,
                        fix_only: false,
                        fix_info: Some(FixInfo {
                            line_number: Some(idx + 1),
                            edit_column: Some(col),
                            delete_count: Some(ial_text.len() as i32),
                            insert_text: None,
                        }),
                        ..Default::default()
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
    use crate::types::RuleParams;
    use std::collections::HashMap;

    fn lint(content: &str) -> Vec<LintError> {
        let lines: Vec<&str> = content.split_inclusive('\n').collect();
        let rule = KMD010;
        rule.lint(&RuleParams {
            name: "test.md",
            version: "0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        })
    }

    #[test]
    fn test_kmd010_valid_class_inline() {
        let errors = lint("# H\n\n*text*{: .highlight}\n");
        assert!(
            errors.is_empty(),
            "valid inline IAL with class should not fire"
        );
    }

    #[test]
    fn test_kmd010_valid_id_inline() {
        let errors = lint("# H\n\n`code`{: #my-id}\n");
        assert!(
            errors.is_empty(),
            "valid inline IAL with id should not fire"
        );
    }

    #[test]
    fn test_kmd010_malformed_inline_ial() {
        let errors = lint("# H\n\n*text*{: bad!!syntax}\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD010")),
            "should fire on malformed inline IAL"
        );
    }

    #[test]
    fn test_kmd010_block_ial_not_flagged() {
        // Block-level IAL (whole line) is handled by KMD006, not this rule
        let errors = lint("# H\n\n{: #my-id}\n");
        assert!(
            errors.is_empty(),
            "block-level IAL should not be flagged by KMD010"
        );
    }

    #[test]
    fn test_kmd010_inside_code_block_ignored() {
        let errors = lint("# H\n\n```\n*text*{: bad!!}\n```\n");
        assert!(errors.is_empty(), "should not fire inside code blocks");
    }

    #[test]
    fn test_kmd010_key_value_inline() {
        let errors = lint("# H\n\n[link](url){: data-x=\"foo\"}\n");
        assert!(
            errors.is_empty(),
            "inline IAL with key=value should not fire"
        );
    }
}
