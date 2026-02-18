//! KMD002 - Footnote references must have matching definitions
//!
//! In Kramdown, footnotes look like:
//! - Reference:   `[^label]`
//! - Definition:  `[^label]: text`
//!
//! This rule fires when a footnote reference has no corresponding definition.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Matches footnote definitions: `[^label]:` at the start of a line
static DEF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[\^([^\]]+)\]:").expect("valid regex"));

/// Matches any `[^label]` occurrence (both refs and defs — we filter in code)
static REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\^([^\]]+)\]").expect("valid regex"));

pub struct KMD002;

impl Rule for KMD002 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD002", "footnote-refs-defined"]
    }

    fn description(&self) -> &'static str {
        "Footnote references must have matching definitions"
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

        // Collect definitions (label → defined)
        let mut definitions: HashSet<String> = HashSet::new();
        // Collect references (label → first line number)
        let mut references: HashMap<String, usize> = HashMap::new();

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

            // Collect definitions
            if let Some(cap) = DEF_RE.captures(line) {
                definitions.insert(cap[1].to_lowercase());
            }

            // Collect references: skip lines that are definitions themselves
            if DEF_RE.is_match(line) {
                // Already counted as a definition above
            } else {
                for cap in REF_RE.captures_iter(line) {
                    let label = cap[1].to_lowercase();
                    references.entry(label).or_insert(idx + 1);
                }
            }
        }

        // Report references without definitions
        let mut undefined: Vec<(String, usize)> = references
            .into_iter()
            .filter(|(label, _)| !definitions.contains(label))
            .collect();
        undefined.sort_by_key(|(_, line)| *line);

        let last_line = lines.len();
        let last_line_len = lines
            .last()
            .map(|l| l.trim_end_matches('\n').trim_end_matches('\r').len())
            .unwrap_or(0);

        for (label, line_number) in undefined {
            errors.push(LintError {
                line_number,
                rule_names: self.names(),
                rule_description: self.description(),
                error_detail: Some(format!("Footnote reference '[^{label}]' has no definition")),
                severity: Severity::Error,
                fix_only: false,
                fix_info: Some(FixInfo {
                    line_number: Some(last_line),
                    edit_column: Some(last_line_len + 1),
                    delete_count: None,
                    insert_text: Some(format!("\n[^{label}]: ")),
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
        let rule = KMD002;
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
    fn test_kmd002_ref_defined_ok() {
        let errors = lint("# H\n\nText[^1] here.\n\n[^1]: The note.\n");
        assert!(errors.is_empty(), "should not fire when ref has definition");
    }

    #[test]
    fn test_kmd002_ref_undefined() {
        let errors = lint("# H\n\nText[^1] here.\n");
        assert!(
            errors.iter().any(|e| e.rule_names.first() == Some(&"KMD002")),
            "should fire when footnote ref is undefined"
        );
    }

    #[test]
    fn test_kmd002_no_footnotes_ok() {
        let errors = lint("# H\n\nPlain paragraph.\n");
        assert!(errors.is_empty(), "should not fire when no footnotes");
    }

    #[test]
    fn test_kmd002_ref_in_code_block_ignored() {
        let errors = lint("# H\n\n```\n[^1] inside code\n```\n");
        assert!(errors.is_empty(), "should not fire for refs in code blocks");
    }

    #[test]
    fn test_kmd002_fix_info_present() {
        let errors = lint("# H\n\nText[^1] here.\n");
        let err = errors.iter().find(|e| e.rule_names.first() == Some(&"KMD002")).unwrap();
        assert!(err.fix_info.is_some(), "KMD002 error should have fix_info");
        let fix = err.fix_info.as_ref().unwrap();
        assert_eq!(fix.insert_text.as_deref(), Some("\n[^1]: "));
        assert!(fix.delete_count.is_none());
    }

    #[test]
    fn test_kmd002_fix_round_trip() {
        use crate::lint::apply_fixes;
        let content = "# H\n\nText[^1] here.\n";
        let errors = lint(content);
        assert!(!errors.is_empty(), "should have KMD002 errors before fix");
        let fixed = apply_fixes(content, &errors);
        let errors2 = lint(&fixed);
        assert!(
            errors2.iter().all(|e| e.rule_names.first() != Some(&"KMD002")),
            "after fix, no KMD002 errors; fixed:\n{fixed}"
        );
    }
}
