//! MD051 - Link fragments should be valid

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use std::sync::LazyLock;

/// Matches same-file fragment links: [text](#fragment)
static FRAGMENT_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(#([^)]+)\)").expect("valid regex"));

/// Matches cross-file fragment links: [text](file.md#fragment)
static CROSS_FILE_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^#)]+)#([^)]+)\)").expect("valid regex"));

pub struct MD051;

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

        // Collect all valid heading IDs for same-file validation
        let heading_ids = crate::helpers::collect_heading_ids(params.lines);

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

            // Same-file fragment links: [text](#fragment)
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

            // Cross-file fragment links: [text](file.md#fragment)
            if let Some(workspace_headings) = params.workspace_headings {
                for cap in CROSS_FILE_LINK_RE.captures_iter(line) {
                    let file_ref = &cap[2];
                    let fragment = &cap[3];

                    // Skip external URLs
                    if file_ref.starts_with("http://") || file_ref.starts_with("https://") {
                        continue;
                    }

                    // Resolve relative path from current file's directory
                    let current_dir = std::path::Path::new(params.name)
                        .parent()
                        .unwrap_or(std::path::Path::new(""));
                    let resolved = current_dir.join(file_ref);

                    // Try to find the target file in the workspace heading index
                    let resolved_str = resolved.to_string_lossy();
                    let target_headings =
                        workspace_headings.get(resolved_str.as_ref()).or_else(|| {
                            // Try canonical path for ../relative resolution
                            resolved.canonicalize().ok().and_then(|p| {
                                workspace_headings.get(&p.to_string_lossy().into_owned())
                            })
                        });

                    if let Some(headings) = target_headings
                        && !headings.contains(&fragment.to_string())
                    {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!(
                                "No matching heading '{}' in '{}'",
                                fragment, file_ref
                            )),
                            error_context: Some(cap[0].to_string()),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: None,
                            suggestion: Some(format!(
                                "Check that '{}' contains a heading that produces anchor '#{}'",
                                file_ref, fragment
                            )),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }
                    // If the target file isn't in workspace_headings, skip silently
                    // (file might not be a .md file or not in workspace)
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
        let ids = crate::helpers::collect_heading_ids(&lines);
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

    #[test]
    fn test_md051_cross_file_valid_fragment() {
        let rule = MD051;
        let lines = vec!["# Local\n", "\n", "[link](other.md#intro)\n"];
        let config = HashMap::new();

        let mut workspace = HashMap::new();
        workspace.insert("other.md".to_string(), vec!["intro".to_string()]);

        let params = crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &config,
            workspace_headings: Some(&workspace),
        };
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md051_cross_file_invalid_fragment() {
        let rule = MD051;
        let lines = vec!["# Local\n", "\n", "[link](other.md#nonexistent)\n"];
        let config = HashMap::new();

        let mut workspace = HashMap::new();
        workspace.insert("other.md".to_string(), vec!["intro".to_string()]);

        let params = crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &config,
            workspace_headings: Some(&workspace),
        };
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("nonexistent")
        );
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("other.md")
        );
    }

    #[test]
    fn test_md051_cross_file_unknown_file_skipped() {
        let rule = MD051;
        let lines = vec!["# Local\n", "\n", "[link](unknown.md#heading)\n"];
        let config = HashMap::new();

        let workspace = HashMap::new(); // empty workspace

        let params = crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &config,
            workspace_headings: Some(&workspace),
        };
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0, "Unknown files should be skipped silently");
    }

    #[test]
    fn test_md051_cross_file_url_skipped() {
        let rule = MD051;
        let lines = vec!["# Local\n", "\n", "[link](https://example.com#fragment)\n"];
        let config = HashMap::new();

        let workspace = HashMap::new();

        let params = crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &config,
            workspace_headings: Some(&workspace),
        };
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0, "URL links should be skipped");
    }

    #[test]
    fn test_md051_cross_file_no_workspace_context() {
        let rule = MD051;
        let lines = vec!["# Local\n", "\n", "[link](other.md#heading)\n"];
        let config = HashMap::new();

        // No workspace context — cross-file validation skipped
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(
            errors.len(),
            0,
            "Cross-file links should be skipped without workspace context"
        );
    }
}
