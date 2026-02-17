//! KMD008 - Block extension syntax must be properly opened and closed
//!
//! Kramdown supports block extensions using the syntax:
//!   `{::name}` … body … `{:/name}`
//!
//! Known extensions:
//!   - `{::comment}…{:/comment}` — treated as a comment, not rendered
//!   - `{::nomarkdown}…{:/nomarkdown}` — content passed through as-is
//!   - `{::options key="val" /}` — self-closing, sets global options
//!
//! This rule fires when an opening `{::name}` has no matching `{:/name}`,
//! when a closing tag has no opener, or when names are mismatched.

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches an opening block extension tag: `{::name}` or `{::name attrs}`
/// Does NOT match self-closing (those end with `/}`).
static OPEN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\{::(\w+)(?:\s[^}]*)?\}$").unwrap());

/// Matches a self-closing block extension: `{::name .../}`
static SELF_CLOSING_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\{::(\w+)[^}]*/\}$").unwrap());

/// Matches a closing block extension tag: `{:/name}`
static CLOSE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\{:/(\w+)\}$").unwrap());

pub struct KMD008;

impl Rule for KMD008 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD008", "block-extension-syntax"]
    }

    fn description(&self) -> &'static str {
        "Block extensions must be properly opened and closed"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "block-extensions"]
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

        // Stack of (name, line_number) for unclosed openers
        let mut stack: Vec<(String, usize)> = Vec::new();
        let mut in_code_block = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r').trim();
            let line_number = idx + 1;

            // Track code fences
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Self-closing tags need no stack management
            if SELF_CLOSING_RE.is_match(trimmed) {
                continue;
            }

            if let Some(cap) = OPEN_RE.captures(trimmed) {
                stack.push((cap[1].to_string(), line_number));
            } else if let Some(cap) = CLOSE_RE.captures(trimmed) {
                let close_name = &cap[1];
                if let Some((open_name, _)) = stack.last() {
                    if open_name == close_name {
                        stack.pop();
                    } else {
                        let open_name = open_name.clone();
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!(
                                "Mismatched block extension: opened '{{::{open_name}}}' but closed with '{{:/{close_name}}}'"
                            )),
                            severity: Severity::Error,
                            ..Default::default()
                        });
                    }
                } else {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!(
                            "Unexpected closing tag '{{:/{close_name}}}' with no matching opening tag"
                        )),
                        severity: Severity::Error,
                        ..Default::default()
                    });
                }
            }
        }

        // Report any unclosed extensions
        for (name, open_line) in stack {
            errors.push(LintError {
                line_number: open_line,
                rule_names: self.names(),
                rule_description: self.description(),
                error_detail: Some(format!(
                    "Unclosed block extension '{{::{name}}}' opened on line {open_line}"
                )),
                severity: Severity::Error,
                ..Default::default()
            });
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
        let rule = KMD008;
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
    fn test_kmd008_matched_comment_ok() {
        let errors = lint("# H\n\n{::comment}\nsome text\n{:/comment}\n");
        assert!(errors.is_empty(), "matched comment block should not fire");
    }

    #[test]
    fn test_kmd008_unclosed_extension() {
        let errors = lint("# H\n\n{::comment}\nsome text\n");
        assert!(
            errors.iter().any(|e| e.rule_names[0] == "KMD008"),
            "should fire on unclosed block extension"
        );
    }

    #[test]
    fn test_kmd008_unexpected_close() {
        let errors = lint("# H\n\n{:/comment}\n");
        assert!(
            errors.iter().any(|e| e.rule_names[0] == "KMD008"),
            "should fire on close tag with no opener"
        );
    }

    #[test]
    fn test_kmd008_mismatched_tags() {
        let errors = lint("# H\n\n{::comment}\ntext\n{:/nomarkdown}\n");
        assert!(
            errors.iter().any(|e| e.rule_names[0] == "KMD008"),
            "should fire on mismatched open/close names"
        );
    }

    #[test]
    fn test_kmd008_self_closing_ok() {
        let errors = lint("# H\n\n{::options auto_ids=\"false\" /}\n\nText\n");
        assert!(errors.is_empty(), "self-closing extension should not fire");
    }

    #[test]
    fn test_kmd008_nomarkdown_ok() {
        let errors = lint("# H\n\n{::nomarkdown}\n<b>raw html</b>\n{:/nomarkdown}\n");
        assert!(
            errors.is_empty(),
            "matched nomarkdown block should not fire"
        );
    }

    #[test]
    fn test_kmd008_inside_code_block_ignored() {
        let errors = lint("# H\n\n```\n{::comment}\ntext\n```\n");
        assert!(errors.is_empty(), "should not fire inside code blocks");
    }
}
