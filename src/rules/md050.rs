//! MD050 - Strong style should be consistent

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD050;

/// Represents a single strong emphasis match in a line
#[derive(Debug)]
struct StrongMatch {
    /// The full matched text including markers (e.g., "**text**" or "__text__")
    full_match: String,
    /// The style of strong emphasis: "asterisk" or "underscore"
    style: String,
    /// 0-based start position in the line
    start: usize,
}

/// Find all strong emphasis patterns in a line.
/// Matches **text** and __text__ but NOT *text* or _text_.
fn find_strong_matches(line: &str) -> Vec<StrongMatch> {
    let mut matches = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i + 1 < len {
        let ch = bytes[i];
        let next = bytes[i + 1];

        if (ch == b'*' && next == b'*') || (ch == b'_' && next == b'_') {
            let marker = ch;

            // Skip tripled markers (e.g., ***)
            if i + 2 < len && bytes[i + 2] == marker {
                i += 1;
                continue;
            }

            let start = i;
            let mut j = i + 2;

            // Content must be non-empty
            if j >= len || bytes[j] == marker || bytes[j] == b'\n' {
                i += 2;
                continue;
            }

            // Find closing double marker
            let mut found_close = false;
            while j + 1 < len {
                if bytes[j] == marker && bytes[j + 1] == marker {
                    // Make sure the closing marker is not tripled
                    let followed_by_marker = j + 2 < len && bytes[j + 2] == marker;

                    if !followed_by_marker {
                        let full = &line[start..j + 2];
                        matches.push(StrongMatch {
                            full_match: full.to_string(),
                            style: if marker == b'*' {
                                "asterisk".to_string()
                            } else {
                                "underscore".to_string()
                            },
                            start,
                        });
                        i = j + 2;
                        found_close = true;
                        break;
                    }
                }
                if bytes[j] == b'\n' {
                    break;
                }
                j += 1;
            }

            if !found_close {
                i += 2;
            }
        } else {
            i += 1;
        }
    }

    matches
}

impl Rule for MD050 {
    fn names(&self) -> &'static [&'static str] {
        &["MD050", "strong-style"]
    }

    fn description(&self) -> &'static str {
        "Strong style should be consistent"
    }

    fn tags(&self) -> &[&'static str] {
        &["emphasis", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md050.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get configured style: "consistent" (default), "asterisk", or "underscore"
        let configured_style = params
            .config
            .get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("consistent")
            .to_string();

        // First pass: collect all strong emphasis occurrences to determine preferred style
        let mut all_matches: Vec<(usize, StrongMatch)> = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            for sm in find_strong_matches(line) {
                all_matches.push((line_number, sm));
            }
        }

        if all_matches.is_empty() {
            return errors;
        }

        // Determine the preferred style
        let preferred_style = if configured_style == "consistent" {
            all_matches[0].1.style.clone()
        } else {
            configured_style.clone()
        };

        // Second pass: report errors for wrong-style strong emphasis with fix_info
        for (line_number, sm) in &all_matches {
            if sm.style != preferred_style {
                let corrected = if preferred_style == "asterisk" {
                    // Replace __text__ with **text**
                    let inner = &sm.full_match[2..sm.full_match.len() - 2];
                    format!("**{}**", inner)
                } else {
                    // Replace **text** with __text__
                    let inner = &sm.full_match[2..sm.full_match.len() - 2];
                    format!("__{}__", inner)
                };

                errors.push(LintError {
                    line_number: *line_number,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!(
                        "Expected: {}; Actual: {}",
                        preferred_style, sm.style
                    )),
                    error_context: Some(sm.full_match.clone()),
                    rule_information: self.information(),
                    error_range: Some((sm.start + 1, sm.full_match.len())),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(sm.start + 1), // 1-based
                        delete_count: Some(sm.full_match.len() as i32),
                        insert_text: Some(corrected),
                    }),
                    suggestion: Some("Use consistent strong emphasis style".to_string()),
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

    #[test]
    fn test_md050_consistent_double_asterisks() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["**bold** text\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md050_consistent_double_underscores() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["__bold__ text\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md050_mixed_styles() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["**bold** and __also bold__\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md050_mixed_styles_consistent_mode() {
        let rule = MD050;
        // First strong is asterisk, so underscore ones should be flagged
        let lines: Vec<&str> = vec!["**one** and __two__\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].error_detail.as_deref(),
            Some("Expected: asterisk; Actual: underscore")
        );
    }

    #[test]
    fn test_md050_configured_asterisk_style() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["__one__ and __two__\n"];
        let tokens = vec![];
        let mut config = HashMap::new();
        config.insert("style".to_string(), serde_json::json!("asterisk"));
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md050_configured_underscore_style() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["**one** and **two**\n"];
        let tokens = vec![];
        let mut config = HashMap::new();
        config.insert("style".to_string(), serde_json::json!("underscore"));
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md050_fix_info_underscore_to_asterisk() {
        let rule = MD050;
        // First is asterisk, so underscore should get fix_info
        let lines: Vec<&str> = vec!["**one** and __two__\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        assert_eq!(fix.line_number, None);
        // "__two__" starts at index 12, 1-based = 13
        assert_eq!(fix.edit_column, Some(13));
        assert_eq!(fix.delete_count, Some(7)); // "__two__".len() == 7
        assert_eq!(fix.insert_text, Some("**two**".to_string()));
    }

    #[test]
    fn test_md050_fix_info_asterisk_to_underscore() {
        let rule = MD050;
        // First is underscore, so asterisk should get fix_info
        let lines: Vec<&str> = vec!["__one__ and **two**\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        // "**two**" starts at index 12, 1-based = 13
        assert_eq!(fix.edit_column, Some(13));
        assert_eq!(fix.delete_count, Some(7));
        assert_eq!(fix.insert_text, Some("__two__".to_string()));
    }

    #[test]
    fn test_md050_fix_info_multiple_errors() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["**ok** and __bad1__ and __bad2__\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);

        let fix0 = errors[0].fix_info.as_ref().expect("first fix_info");
        assert_eq!(fix0.insert_text, Some("**bad1**".to_string()));

        let fix1 = errors[1].fix_info.as_ref().expect("second fix_info");
        assert_eq!(fix1.insert_text, Some("**bad2**".to_string()));
    }

    #[test]
    fn test_md050_no_strong() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["Just plain text.\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md050_multiline() {
        let rule = MD050;
        let lines: Vec<&str> = vec!["**first** line\n", "__second__ line\n"];
        let tokens = vec![];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test_with_tokens(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(10)); // "__second__".len() == 10
        assert_eq!(fix.insert_text, Some("**second**".to_string()));
    }
}
