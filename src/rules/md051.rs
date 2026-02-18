//! MD051 - Link fragments should be valid

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static FRAGMENT_LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([^\]]*)\]\(#([^)]+)\)").expect("valid regex"));

pub struct MD051;

/// Collect all heading IDs with duplicate handling
fn collect_heading_ids(lines: &[&str]) -> Vec<String> {
    let mut ids = Vec::new();
    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for heading in crate::helpers::parse_headings(lines) {
        let base_id = crate::helpers::heading_to_anchor_id(&heading.text);
        let count = id_counts.entry(base_id.clone()).or_insert(0);
        let final_id = if *count == 0 {
            base_id
        } else {
            format!("{}-{}", base_id, count)
        };
        *count += 1;
        ids.push(final_id);
    }

    ids
}

impl Rule for MD051 {
    fn names(&self) -> &'static [&'static str] {
        &["MD051", "link-fragments"]
    }

    fn description(&self) -> &'static str {
        "Link fragments should be valid"
    }

    fn tags(&self) -> &[&'static str] {
        &["links"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md051.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Collect all valid heading IDs
        let heading_ids = collect_heading_ids(params.lines);

        // Find all fragment links and check them
        let mut in_code_block = false;
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if crate::helpers::is_code_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            for cap in FRAGMENT_LINK_RE.captures_iter(line) {
                let fragment = &cap[2];
                if !heading_ids.contains(&fragment.to_string()) {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!(
                            "No matching heading for fragment: #{}",
                            fragment
                        )),
                        error_context: Some(cap[0].to_string()),
                        rule_information: self.information(),
                        error_range: None,
                        fix_info: None,
                        suggestion: Some(
                            "Ensure link fragments point to valid headings".to_string(),
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
    fn test_collect_heading_ids_duplicates() {
        let lines = vec!["# Title\n", "## Section\n", "## Section\n", "## Section\n"];
        let ids = collect_heading_ids(&lines);
        assert_eq!(ids, vec!["title", "section", "section-1", "section-2"]);
    }

    #[test]
    fn test_md051_valid_fragment() {
        let rule = MD051;
        let lines = vec![
            "# Title\n",
            "\n",
            "## Getting Started\n",
            "\n",
            "See [title](#title) and [start](#getting-started).\n",
        ];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md051_invalid_fragment() {
        let rule = MD051;
        let lines = vec!["# Title\n", "\n", "See [missing](#nonexistent).\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("nonexistent")
        );
    }

    #[test]
    fn test_md051_duplicate_heading_ids() {
        let rule = MD051;
        let lines = vec![
            "# Title\n",
            "## Section\n",
            "## Section\n",
            "\n",
            "See [first](#section) and [second](#section-1).\n",
        ];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md051_fragment_in_code_block_ignored() {
        let rule = MD051;
        let lines = vec!["# Title\n", "```\n", "[link](#nonexistent)\n", "```\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md051_multiple_fragments_one_line() {
        let rule = MD051;
        let lines = vec![
            "# Title\n",
            "## About\n",
            "\n",
            "See [a](#title) and [b](#missing).\n",
        ];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md051_punctuation_only_heading_no_panic() {
        // Heading with only punctuation produces empty ID after stripping
        let rule = MD051;
        let lines = vec!["## ???\n", "\n", "[link](#)\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        // Should not panic — just produce an error for invalid fragment
        let _errors = rule.lint(&params);
    }

    #[test]
    fn test_md051_unicode_heading() {
        let rule = MD051;
        let lines = vec![
            "# Caf\u{00e9} Guide\n",
            "\n",
            "[link](#caf\u{00e9}-guide)\n",
        ];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0, "Unicode heading IDs should match");
    }
}
