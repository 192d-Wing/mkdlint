//! MD053 - Link and image reference definitions should be needed

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// Regex for reference link definitions: `[label]: url`
static DEF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\[([^\]]+)\]:\s+").expect("valid regex"));

/// Regex for full reference links: `[text][label]`
static FULL_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]*)\]\[([^\]]+)\]").expect("valid regex"));

/// Regex for collapsed reference links: `[label][]`
static COLLAPSED_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\[\]").expect("valid regex"));

/// Regex for shortcut reference links: `[label]` (not followed by `[` or `(` or `:`)
static SHORTCUT_REF_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([^\]]+)\](?:[^(\[:]|$)").expect("valid regex"));

pub struct MD053;

/// Check if a line is a code fence opener/closer (``` or ~~~)
fn is_code_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// Check if a label matches any of the ignored patterns
fn is_ignored(label: &str, ignored_definitions: &[String]) -> bool {
    ignored_definitions.iter().any(|pattern| pattern == label)
}

impl Rule for MD053 {
    fn names(&self) -> &'static [&'static str] {
        &["MD053", "link-image-reference-definitions"]
    }

    fn description(&self) -> &'static str {
        "Link and image reference definitions should be needed"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "images", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md053.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Read ignored_definitions from config, default to ["//"]
        let ignored_definitions: Vec<String> = params
            .config
            .get("ignored_definitions")
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
            .unwrap_or_else(|| vec!["//".to_string()]);

        // Pass 1: Collect all reference definitions with line numbers (skipping code blocks)
        let mut definitions: Vec<(String, usize)> = Vec::new(); // (label_lowercase, line_number)
        let mut in_code_block = false;
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            if is_code_fence(line) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            if let Some(caps) = DEF_RE.captures(line) {
                let label = caps[1].to_string();
                let label_lower = label.to_lowercase();

                // Skip ignored definitions
                if is_ignored(
                    &label_lower,
                    &ignored_definitions
                        .iter()
                        .map(|s| s.to_lowercase())
                        .collect::<Vec<_>>(),
                ) {
                    continue;
                }

                definitions.push((label_lower, line_number));
            }
        }

        // Pass 2: Collect all reference usages (skipping code blocks)
        let mut used_labels: HashSet<String> = HashSet::new();
        in_code_block = false;
        for line in params.lines.iter() {
            if is_code_fence(line) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Full reference links: [text][label]
            for caps in FULL_REF_RE.captures_iter(line) {
                used_labels.insert(caps[2].to_lowercase());
            }

            // Collapsed reference links: [label][]
            for caps in COLLAPSED_REF_RE.captures_iter(line) {
                used_labels.insert(caps[1].to_lowercase());
            }

            // Shortcut reference links: [label]
            for caps in SHORTCUT_REF_RE.captures_iter(line) {
                used_labels.insert(caps[1].to_lowercase());
            }
        }

        // Report definitions that are never used
        for (label, line_number) in &definitions {
            if !used_labels.contains(label) {
                errors.push(LintError {
                    line_number: *line_number,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!("Unused reference definition \"{}\"", label)),
                    error_context: None,
                    rule_information: self.information(),
                    error_range: None,
                    fix_info: Some(FixInfo {
                        line_number: Some(*line_number),
                        edit_column: Some(1),
                        delete_count: Some(-1), // Delete entire line
                        insert_text: None,
                    }),
                    suggestion: Some("Remove this unused link definition".to_string()),
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

    fn make_params<'a>(
        lines: &'a [&'a str],
        config: &'a HashMap<String, serde_json::Value>,
    ) -> RuleParams<'a> {
        RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens: &[],
            config,
        }
    }

    #[test]
    fn test_md053_all_used() {
        let lines: Vec<&str> = vec![
            "This has a [link][foo] reference.\n",
            "\n",
            "[foo]: https://example.com\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD053;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md053_unused_definition() {
        let lines: Vec<&str> = vec!["This is some text.\n", "\n", "[foo]: https://example.com\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD053;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
    }

    #[test]
    fn test_md053_ignored_definition() {
        let lines: Vec<&str> = vec!["This is some text.\n", "\n", "[//]: https://example.com\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD053;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md053_fix_unused_definition() {
        let lines: Vec<&str> = vec!["This is some text.\n", "\n", "[foo]: https://example.com\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD053;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());

        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.line_number, Some(3));
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(-1)); // Delete entire line
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md053_fix_multiple_unused() {
        let lines: Vec<&str> = vec![
            "This has a [link][bar] reference.\n",
            "\n",
            "[foo]: https://example.com\n",
            "[bar]: https://example.org\n",
            "[baz]: https://example.net\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD053;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2); // foo and baz are unused

        // Both should have fix_info
        for error in &errors {
            assert!(error.fix_info.is_some());
            let fix = error.fix_info.as_ref().unwrap();
            assert_eq!(fix.delete_count, Some(-1));
        }
    }

    #[test]
    fn test_md053_no_fix_for_used() {
        let lines: Vec<&str> = vec![
            "This has a [link][foo] reference.\n",
            "\n",
            "[foo]: https://example.com\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD053;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0); // No errors, all definitions used
    }
}
