//! MD042 - No empty links
//!
//! This rule checks for links with no URL or only a fragment (#).

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static INLINE_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Match inline links: [text](url)
    // Captures the link text and the URL part
    Regex::new(r"\[([^\]]*)\]\(([^)]*)\)").expect("valid regex")
});

static REFERENCE_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Match reference links: [text][ref] or [text][] or [text]
    // Note: We can't use negative lookahead (?!\() in Rust regex, so we'll filter inline links manually
    Regex::new(r"\[([^\]]+)\](?:\[([^\]]*)\])?").expect("valid regex")
});

static LINK_DEFINITION_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Match link definitions: [ref]: url
    // Note: no $ anchor because lines may have trailing \n
    Regex::new(r"^\s*\[([^\]]+)\]:\s*(\S*)").expect("valid regex")
});

pub struct MD042;

impl MD042 {
    /// Check if a URL is empty or just a fragment
    fn is_empty_or_fragment_only(url: &str) -> bool {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return true;
        }

        // Check for empty angle brackets: <> or < >
        let angle_stripped = trimmed.trim_start_matches('<').trim_end_matches('>').trim();
        if angle_stripped.is_empty() {
            return true;
        }

        // Check for just # or # with title
        if let Some(after_hash) = trimmed.strip_prefix('#') {
            let after_hash = after_hash.trim();
            // If nothing after # or if it starts with a quote (title), it's empty
            if after_hash.is_empty() || after_hash.starts_with('"') || after_hash.starts_with('\'')
            {
                return true;
            }
        }

        false
    }
}

impl Rule for MD042 {
    fn names(&self) -> &'static [&'static str] {
        &["MD042", "no-empty-links"]
    }

    fn description(&self) -> &'static str {
        "No empty links"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md042.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // First pass: collect link definitions
        let mut definitions: HashMap<String, String> = HashMap::new();
        for line in params.lines.iter() {
            if let Some(cap) = LINK_DEFINITION_RE.captures(line) {
                let ref_name = cap.get(1).unwrap().as_str().trim().to_lowercase();
                let url = cap.get(2).unwrap().as_str().trim().to_string();
                definitions.insert(ref_name, url);
            }
        }

        // Second pass: check links
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            // Skip link definition lines
            if LINK_DEFINITION_RE.is_match(line) {
                continue;
            }

            // Check inline links
            for cap in INLINE_LINK_RE.captures_iter(line) {
                let full_match = cap.get(0).unwrap();
                let url = cap.get(2).unwrap().as_str();

                if Self::is_empty_or_fragment_only(url) {
                    // Calculate position for fix
                    let paren_content = cap.get(2).unwrap();
                    let url_start = paren_content.start();
                    let url_col = url_start + 1; // 1-based column

                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: None,
                        error_context: Some(full_match.as_str().to_string()),
                        rule_information: self.information(),
                        error_range: Some((full_match.start() + 1, full_match.len())),
                        fix_info: Some(FixInfo {
                            line_number: None,
                            edit_column: Some(url_col),
                            delete_count: Some(url.len() as i32),
                            insert_text: Some("#link".to_string()),
                        }),
                        suggestion: Some(
                            "Provide a URL or use '#' as a placeholder for the link destination"
                                .to_string(),
                        ),
                        severity: Severity::Error,
                        fix_only: false,
                    });
                }
            }

            // Check reference links
            for cap in REFERENCE_LINK_RE.captures_iter(line) {
                let full_match = cap.get(0).unwrap();

                // Skip if this is actually an inline link (followed by '(')
                let end_pos = full_match.end();
                if line.as_bytes().get(end_pos) == Some(&b'(') {
                    continue;
                }

                let text = cap.get(1).unwrap().as_str();
                let ref_name = if let Some(r) = cap.get(2) {
                    let ref_str = r.as_str();
                    if ref_str.is_empty() {
                        // [text][] form - use text as reference
                        text
                    } else {
                        ref_str
                    }
                } else {
                    // [text] form - use text as reference
                    text
                };

                let ref_key = ref_name.trim().to_lowercase();

                // Check if this reference exists and if it points to an empty URL
                if let Some(url) = definitions.get(&ref_key)
                    && Self::is_empty_or_fragment_only(url)
                {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: None,
                        error_context: Some(full_match.as_str().to_string()),
                        rule_information: self.information(),
                        error_range: Some((full_match.start() + 1, full_match.len())),
                        fix_info: None,
                        suggestion: None,
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
    fn test_md042_empty_inline_link() {
        let lines = vec!["[text]()\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
    }

    #[test]
    fn test_md042_empty_with_angle_brackets() {
        let lines = vec!["[text](<>)\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md042_fragment_only() {
        let lines = vec!["[text](#)\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md042_fragment_with_title() {
        let lines = vec!["[text](# \"title\")\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md042_valid_link() {
        let lines = vec!["[text](https://example.com)\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md042_valid_fragment() {
        let lines = vec!["[text](#section)\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md042_reference_link_with_empty_definition() {
        let lines = vec!["[text][frag]\n", "\n", "[frag]: #\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
    }

    #[test]
    fn test_md042_reference_link_shorthand() {
        let lines = vec!["[frag][]\n", "\n", "[frag]: #\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md042_reference_link_implicit() {
        let lines = vec!["[frag]\n", "\n", "[frag]: #\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md042_reference_link_with_valid_definition() {
        let lines = vec!["[text][ref]\n", "\n", "[ref]: https://example.com\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md042_multiple_empty_links_on_same_line() {
        let lines = vec!["[text1](link-1) [text2]() [text3](link-3)\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .error_context
                .as_ref()
                .unwrap()
                .contains("[text2]()")
        );
    }

    #[test]
    fn test_md042_fix_empty_inline_link() {
        let lines = vec!["[text]()\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(8)); // After "("
        assert_eq!(fix.delete_count, Some(0)); // Empty URL
        assert_eq!(fix.insert_text, Some("#link".to_string()));
    }

    #[test]
    fn test_md042_fix_fragment_only() {
        let lines = vec!["[text](#)\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(8)); // After "("
        assert_eq!(fix.delete_count, Some(1)); // Delete "#"
        assert_eq!(fix.insert_text, Some("#link".to_string()));
    }

    #[test]
    fn test_md042_no_fix_reference_link() {
        let lines = vec!["[text][frag]\n", "\n", "[frag]: #\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD042;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        // Reference links should not have fix_info
        assert!(errors[0].fix_info.is_none());
    }
}
