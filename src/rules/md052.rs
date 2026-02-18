//! MD052 - Reference links and images should use a label that is defined

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// Regex for reference link definitions: `[label]: url`
static DEF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\[([^\]]+)\]:\s+").unwrap());

/// Regex for full reference links: `[text][label]`
static FULL_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]*)\]\[([^\]]+)\]").unwrap());

/// Regex for collapsed reference links: `[label][]`
static COLLAPSED_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\[\]").unwrap());

pub struct MD052;

/// Check if a line is a code fence opener/closer (``` or ~~~)
fn is_code_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

impl Rule for MD052 {
    fn names(&self) -> &'static [&'static str] {
        &["MD052", "reference-links-images"]
    }

    fn description(&self) -> &'static str {
        "Reference links and images should use a label that is defined"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "images", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md052.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut defined_labels: HashSet<String> = HashSet::new();

        // Pass 1: Collect all reference definitions (skipping code blocks)
        let mut in_code_block = false;
        for line in params.lines.iter() {
            if is_code_fence(line) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            if let Some(caps) = DEF_RE.captures(line) {
                let label = caps[1].to_lowercase();
                defined_labels.insert(label);
            }
        }

        // Pass 2: Find all reference usages and check if they are defined
        in_code_block = false;
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            if is_code_fence(line) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Check full reference links: [text][label]
            for caps in FULL_REF_RE.captures_iter(line) {
                let label = caps[2].to_lowercase();
                if !defined_labels.contains(&label) {
                    // Append to the last non-empty line
                    // Note: apply_fixes pops trailing empty lines (lines that are just "\n" or "\r\n")
                    // so we need to target the line before it if it exists
                    let last_line_idx = params.lines.len().saturating_sub(1);
                    let is_trailing_empty = params
                        .lines
                        .get(last_line_idx)
                        .map(|l| *l == "\n" || *l == "\r\n")
                        .unwrap_or(false);
                    let insert_line = if is_trailing_empty {
                        last_line_idx.max(1) // Target line before trailing empty
                    } else {
                        params.lines.len() // Target the actual last line
                    };
                    let target_line = params.lines.get(insert_line - 1).copied().unwrap_or("");
                    let target_stripped = target_line.trim_end_matches('\n').trim_end_matches('\r');
                    let insert_col = target_stripped.len() + 1;

                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!(
                            "Reference label \"{}\" is not defined",
                            &caps[2]
                        )),
                        error_context: Some(caps[0].to_string()),
                        rule_information: self.information(),
                        error_range: None,
                        fix_info: Some(crate::types::FixInfo {
                            line_number: Some(insert_line),
                            edit_column: Some(insert_col),
                            delete_count: None,
                            insert_text: Some(format!("\n[{}]: #link\n", &caps[2])),
                        }),
                        suggestion: Some(
                            "Define all link reference labels that are used".to_string(),
                        ),
                        severity: Severity::Error,
                        fix_only: false,
                    });
                }
            }

            // Check collapsed reference links: [label][]
            for caps in COLLAPSED_REF_RE.captures_iter(line) {
                let label = caps[1].to_lowercase();
                if !defined_labels.contains(&label) {
                    // Append to the last non-empty line
                    // Note: apply_fixes pops trailing empty lines (lines that are just "\n" or "\r\n")
                    // so we need to target the line before it if it exists
                    let last_line_idx = params.lines.len().saturating_sub(1);
                    let is_trailing_empty = params
                        .lines
                        .get(last_line_idx)
                        .map(|l| *l == "\n" || *l == "\r\n")
                        .unwrap_or(false);
                    let insert_line = if is_trailing_empty {
                        last_line_idx.max(1) // Target line before trailing empty
                    } else {
                        params.lines.len() // Target the actual last line
                    };
                    let target_line = params.lines.get(insert_line - 1).copied().unwrap_or("");
                    let target_stripped = target_line.trim_end_matches('\n').trim_end_matches('\r');
                    let insert_col = target_stripped.len() + 1;

                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!(
                            "Reference label \"{}\" is not defined",
                            &caps[1]
                        )),
                        error_context: Some(caps[0].to_string()),
                        rule_information: self.information(),
                        error_range: None,
                        fix_info: Some(crate::types::FixInfo {
                            line_number: Some(insert_line),
                            edit_column: Some(insert_col),
                            delete_count: None,
                            insert_text: Some(format!("\n[{}]: #link\n", &caps[1])),
                        }),
                        suggestion: Some(
                            "Define all link reference labels that are used".to_string(),
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
    fn test_md052_valid_references() {
        let lines: Vec<&str> = vec![
            "This has a [link][foo] reference.\n",
            "\n",
            "[foo]: https://example.com\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md052_undefined_reference() {
        let lines: Vec<&str> = vec![
            "This has a [link][bar] reference.\n",
            "\n",
            "[foo]: https://example.com\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
    }

    #[test]
    fn test_md052_case_insensitive() {
        let lines: Vec<&str> = vec![
            "This has a [link][foo] reference.\n",
            "\n",
            "[Foo]: https://example.com\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md052_fix_full_reference() {
        let lines: Vec<&str> = vec![
            "This has a [link][bar] reference.\n",
            "\n",
            "[foo]: https://example.com\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);

        let fix_info = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix_info.line_number, Some(3));
        assert!(
            fix_info
                .insert_text
                .as_ref()
                .unwrap()
                .contains("[bar]: #link")
        );
    }

    #[test]
    fn test_md052_fix_collapsed_reference() {
        let lines: Vec<&str> = vec![
            "This has a [link][] reference.\n",
            "\n",
            "[foo]: https://example.com\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);

        let fix_info = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix_info.line_number, Some(3));
        assert!(
            fix_info
                .insert_text
                .as_ref()
                .unwrap()
                .contains("[link]: #link")
        );
    }

    #[test]
    fn test_md052_fix_multiple_undefined() {
        let lines: Vec<&str> = vec!["This has [link1][ref1] and [link2][ref2].\n", "\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);

        // Both should have fix_info
        assert!(errors[0].fix_info.is_some());
        assert!(errors[1].fix_info.is_some());
    }

    #[test]
    fn test_md052_fix_integration() {
        use crate::apply_fixes;

        let content = "# Title\n\nSee [link][foo].\n";
        // Simulate CLI line splitting (same as lint_content)
        let lines: Vec<&str> = vec!["# Title\n", "\n", "See [link][foo].\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        println!("Content: {:?}", content);
        println!("Lines ({}): {:?}", lines.len(), lines);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        // Debug fix_info
        let fix_info = errors[0].fix_info.as_ref().unwrap();
        println!(
            "Fix info: line_number={:?}, edit_column={:?}, insert_text={:?}",
            fix_info.line_number, fix_info.edit_column, fix_info.insert_text
        );

        // Apply the fix (use original content, not lines)
        let fixed = apply_fixes(content, &errors);
        println!("Original (len={}):\n{:?}", content.len(), content);
        println!("Fixed (len={}):\n{:?}", fixed.len(), fixed);
        println!("Changed: {}", fixed != content);

        // The fixed content should contain the reference definition
        assert!(
            fixed.contains("[foo]: #link"),
            "Fixed content should contain reference definition"
        );
    }
}
