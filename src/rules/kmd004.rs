//! KMD004 - Abbreviation definitions should be used in document text
//!
//! In Kramdown, abbreviations are defined with:
//!   `*[ABBR]: expansion text`
//!
//! This rule fires when an abbreviation is defined but the abbreviation term
//! never appears in the document body.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches abbreviation definitions: `*[TERM]: expansion`
static ABBR_DEF_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\*\[([^\]]+)\]:").expect("valid regex"));

pub struct KMD004;

impl Rule for KMD004 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD004", "abbreviation-defs-used"]
    }

    fn description(&self) -> &'static str {
        "Abbreviation definitions should be used in document text"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "abbreviations", "fixable"]
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

        // Collect abbreviation definitions: term → line number
        let mut abbreviations: Vec<(String, usize)> = Vec::new();
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

            if let Some(cap) = ABBR_DEF_RE.captures(line) {
                abbreviations.push((cap[1].to_string(), idx + 1));
            }
        }

        if abbreviations.is_empty() {
            return errors;
        }

        // Build the full document text (excluding abbreviation definition lines)
        let body: String = lines
            .iter()
            .filter(|line| !ABBR_DEF_RE.is_match(line))
            .map(|l| l.trim_end_matches('\n').trim_end_matches('\r'))
            .collect::<Vec<_>>()
            .join("\n");

        for (term, line_number) in abbreviations {
            if !body.contains(term.as_str()) {
                errors.push(LintError {
                    line_number,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!(
                        "Abbreviation '{term}' is defined but never used in text"
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
        let rule = KMD004;
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
    fn test_kmd004_abbr_used_ok() {
        let errors = lint("# H\n\nHTML is great.\n\n*[HTML]: HyperText Markup Language\n");
        assert!(
            errors.is_empty(),
            "should not fire when abbreviation is used"
        );
    }

    #[test]
    fn test_kmd004_abbr_unused() {
        let errors = lint("# H\n\nSome text.\n\n*[HTML]: HyperText Markup Language\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD004")),
            "should fire when abbreviation is never used"
        );
    }

    #[test]
    fn test_kmd004_no_abbr_ok() {
        let errors = lint("# H\n\nPlain paragraph.\n");
        assert!(errors.is_empty(), "should not fire when no abbreviations");
    }

    #[test]
    fn test_kmd004_fix_info_present() {
        let errors = lint("# H\n\nSome text.\n\n*[HTML]: HyperText Markup Language\n");
        let err = errors
            .iter()
            .find(|e| e.rule_names.first() == Some(&"KMD004"))
            .unwrap();
        assert!(err.fix_info.is_some(), "KMD004 error should have fix_info");
        let fix = err.fix_info.as_ref().unwrap();
        assert_eq!(fix.delete_count, Some(-1));
        assert!(fix.insert_text.is_none());
    }

    #[test]
    fn test_kmd004_fix_round_trip() {
        use crate::lint::apply_fixes;
        let content = "# H\n\nSome text.\n\n*[HTML]: HyperText Markup Language\n";
        let errors = lint(content);
        assert!(!errors.is_empty(), "should have KMD004 errors before fix");
        let fixed = apply_fixes(content, &errors);
        let errors2 = lint(&fixed);
        assert!(
            errors2
                .iter()
                .all(|e| e.rule_names.first() != Some(&"KMD004")),
            "after fix, no KMD004 errors; fixed:\n{fixed}"
        );
    }
}
