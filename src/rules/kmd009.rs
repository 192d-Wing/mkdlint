//! KMD009 - Attribute List Definition (ALD) entries must be referenced
//!
//! In Kramdown, an ALD defines a reusable set of attributes:
//!   `{:ref-name: #id .class key="value"}`
//!
//! The ALD can then be applied to block or span elements by referencing it:
//!   `{: ref-name}` or `{: ref-name .extra-class}`
//!
//! This rule fires when an ALD is defined but never referenced in the document.
//!
//! ## Distinguishing ALDs from IALs
//!
//! Both start with `{:`. The difference is:
//! - ALD definition: `{:identifier: ...}` — identifier immediately followed by `:`
//! - Regular IAL:    `{: #id .class ...}` — starts with space, `#`, `.`, or `key=`

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Matches an ALD definition: `{:identifier: attrs}` at start of line.
/// Captures the identifier name.
static ALD_DEF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\{:([A-Za-z][\w-]*):\s").expect("valid regex"));

/// Matches an ALD reference: `{:identifier}` anywhere in a line.
/// Captures the identifier name.
static ALD_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{:([A-Za-z][\w-]*)\}").expect("valid regex"));

pub struct KMD009;

impl Rule for KMD009 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD009", "ald-defs-used"]
    }

    fn description(&self) -> &'static str {
        "Attribute List Definitions must be referenced in the document"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "ald", "attributes", "fixable"]
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

        // identifier → first line number
        let mut definitions: HashMap<String, usize> = HashMap::new();
        let mut references: std::collections::HashSet<String> = std::collections::HashSet::new();

        let mut in_code_block = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Track code fences
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Collect ALD definitions
            if let Some(cap) = ALD_DEF_RE.captures(trimmed) {
                definitions.entry(cap[1].to_string()).or_insert(idx + 1);
                // Don't collect references from definition lines
                continue;
            }

            // Collect ALD references
            for cap in ALD_REF_RE.captures_iter(trimmed) {
                references.insert(cap[1].to_string());
            }
        }

        // Report definitions without references
        let mut unused: Vec<(String, usize)> = definitions
            .into_iter()
            .filter(|(name, _)| !references.contains(name))
            .collect();
        unused.sort_by_key(|(_, line)| *line);

        for (name, line_number) in unused {
            errors.push(LintError {
                line_number,
                rule_names: self.names(),
                rule_description: self.description(),
                error_detail: Some(format!(
                    "ALD '{{:{name}:}}' is defined but never referenced"
                )),
                severity: Severity::Error,
                fix_only: false,
                fix_info: Some(FixInfo {
                    line_number: Some(line_number),
                    edit_column: Some(1),
                    delete_count: Some(-1), // Delete entire line
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
        let rule = KMD009;
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
    fn test_kmd009_ald_referenced_ok() {
        let errors = lint("# H\n\n{:myref: .highlight}\n\nA paragraph\n{:myref}\n");
        assert!(errors.is_empty(), "referenced ALD should not fire");
    }

    #[test]
    fn test_kmd009_ald_unused() {
        let errors = lint("# H\n\n{:myref: .highlight}\n\nA paragraph.\n");
        assert!(
            errors.iter().any(|e| e.rule_names.first() == Some(&"KMD009")),
            "should fire when ALD is never referenced"
        );
    }

    #[test]
    fn test_kmd009_no_ald_ok() {
        let errors = lint("# H\n\nPlain paragraph.\n");
        assert!(errors.is_empty(), "no ALDs should not fire");
    }

    #[test]
    fn test_kmd009_ial_not_confused_with_ald() {
        // Regular IAL starting with #, ., or space should not be treated as ALD
        let errors = lint("# H\n\n{: #my-id .class}\n\nText\n");
        assert!(
            errors.is_empty(),
            "regular IAL should not be treated as ALD"
        );
    }

    #[test]
    fn test_kmd009_inside_code_block_ignored() {
        let errors = lint("# H\n\n```\n{:myref: .class}\n```\n");
        assert!(errors.is_empty(), "should not fire inside code blocks");
    }

    #[test]
    fn test_kmd009_fix_info_present() {
        let errors = lint("# H\n\n{:myref: .highlight}\n\nA paragraph.\n");
        let err = errors.iter().find(|e| e.rule_names.first() == Some(&"KMD009")).unwrap();
        assert!(err.fix_info.is_some(), "KMD009 error should have fix_info");
        let fix = err.fix_info.as_ref().unwrap();
        assert_eq!(fix.delete_count, Some(-1));
        assert!(fix.insert_text.is_none());
    }

    #[test]
    fn test_kmd009_fix_round_trip() {
        use crate::lint::apply_fixes;
        let content = "# H\n\n{:myref: .highlight}\n\nA paragraph.\n";
        let errors = lint(content);
        assert!(!errors.is_empty(), "should have KMD009 errors before fix");
        let fixed = apply_fixes(content, &errors);
        let errors2 = lint(&fixed);
        assert!(
            errors2.iter().all(|e| e.rule_names.first() != Some(&"KMD009")),
            "after fix, no KMD009 errors; fixed:\n{fixed}"
        );
    }
}
