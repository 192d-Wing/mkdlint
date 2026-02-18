//! KMD003 - Footnote definitions must be referenced
//!
//! In Kramdown, footnote definitions that are never referenced add noise.
//! This rule fires when a `[^label]:` definition has no corresponding `[^label]` reference.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Matches footnote definitions: `[^label]:` at the start of a line
static DEF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[\^([^\]]+)\]:").expect("valid regex"));

/// Matches any `[^label]` occurrence (both refs and defs — we filter in code)
static REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\^([^\]]+)\]").expect("valid regex"));

pub struct KMD003;

impl Rule for KMD003 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD003", "footnote-defs-used"]
    }

    fn description(&self) -> &'static str {
        "Footnote definitions must be referenced in the document"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "footnotes", "fixable"]
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

        // Collect definitions (label → line number)
        let mut definitions: HashMap<String, usize> = HashMap::new();
        // Collect references
        let mut references: HashSet<String> = HashSet::new();

        let mut in_code_block = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Track code fences
            if crate::helpers::is_code_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Collect definitions
            if let Some(cap) = DEF_RE.captures(line) {
                definitions.entry(cap[1].to_lowercase()).or_insert(idx + 1);
            }

            // Collect references: skip lines that are definitions themselves
            if !DEF_RE.is_match(line) {
                for cap in REF_RE.captures_iter(line) {
                    references.insert(cap[1].to_lowercase());
                }
            }
        }

        // Report definitions without references
        let mut unused: Vec<(String, usize)> = definitions
            .into_iter()
            .filter(|(label, _)| !references.contains(label))
            .collect();
        unused.sort_by_key(|(_, line)| *line);

        for (label, line_number) in unused {
            errors.push(LintError {
                line_number,
                rule_names: self.names(),
                rule_description: self.description(),
                error_detail: Some(format!(
                    "Footnote definition '[^{label}]' is never referenced"
                )),
                severity: Severity::Error,
                fix_only: false,
                fix_info: Some(FixInfo {
                    line_number: Some(line_number),
                    edit_column: Some(1),
                    delete_count: Some(-1),
                    insert_text: None,
                }),
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
        let rule = KMD003;
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
    fn test_kmd003_def_used_ok() {
        let errors = lint("# H\n\nText[^1] here.\n\n[^1]: The note.\n");
        assert!(errors.is_empty(), "should not fire when def is referenced");
    }

    #[test]
    fn test_kmd003_def_unused() {
        let errors = lint("# H\n\nText here.\n\n[^1]: An unused note.\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD003")),
            "should fire when footnote def is never referenced"
        );
    }

    #[test]
    fn test_kmd003_no_footnotes_ok() {
        let errors = lint("# H\n\nPlain paragraph.\n");
        assert!(errors.is_empty(), "should not fire when no footnotes");
    }

    #[test]
    fn test_kmd003_def_in_code_block_ignored() {
        let errors = lint("# H\n\n```\n[^1]: inside code\n```\n");
        assert!(errors.is_empty(), "should not fire for defs in code blocks");
    }

    #[test]
    fn test_kmd003_fix_info_present() {
        let errors = lint("# H\n\nText here.\n\n[^1]: An unused note.\n");
        let err = errors
            .iter()
            .find(|e| e.rule_names.first() == Some(&"KMD003"))
            .unwrap();
        assert!(err.fix_info.is_some(), "KMD003 error should have fix_info");
        let fix = err.fix_info.as_ref().unwrap();
        assert_eq!(fix.delete_count, Some(-1));
        assert!(fix.insert_text.is_none());
    }

    #[test]
    fn test_kmd003_fix_round_trip() {
        use crate::lint::apply_fixes;
        let content = "# H\n\nText here.\n\n[^1]: An unused note.\n";
        let errors = lint(content);
        assert!(!errors.is_empty(), "should have KMD003 errors before fix");
        let fixed = apply_fixes(content, &errors);
        let errors2 = lint(&fixed);
        assert!(
            errors2
                .iter()
                .all(|e| e.rule_names.first() != Some(&"KMD003")),
            "after fix, no KMD003 errors; fixed:\n{fixed}"
        );
    }
}
