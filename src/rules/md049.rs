//! MD049 - Emphasis style should be consistent

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD049;

/// Represents a single emphasis match in a line
#[derive(Debug)]
struct EmphasisMatch {
    /// The full matched text including markers (e.g., "*text*" or "_text_")
    full_match: String,
    /// The style of emphasis: "asterisk" or "underscore"
    style: String,
    /// 0-based start position in the line
    start: usize,
}

/// Find all single-emphasis patterns in a line.
/// Matches *text* and _text_ but NOT **text** or __text__.
fn find_emphasis_matches(line: &str) -> Vec<EmphasisMatch> {
    let mut matches = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len {
        let ch = bytes[i];

        if ch == b'*' || ch == b'_' {
            // Skip if this is a doubled marker (strong emphasis)
            if i + 1 < len && bytes[i + 1] == ch {
                // This is ** or __, skip the strong emphasis block entirely
                // Find the closing ** or __
                let marker = ch;
                let mut j = i + 2;
                while j + 1 < len {
                    if bytes[j] == marker && bytes[j + 1] == marker {
                        // Check it's not tripled (or more) at the start
                        j += 2;
                        break;
                    }
                    j += 1;
                }
                i = j;
                continue;
            }

            // Single marker -- look for closing single marker
            let marker = ch;
            let start = i;
            let mut j = i + 1;

            // Content must be non-empty and not start with a space
            if j >= len || bytes[j] == b' ' || bytes[j] == b'\n' || bytes[j] == marker {
                i += 1;
                continue;
            }

            // Find closing single marker (not doubled)
            let mut found_close = false;
            while j < len {
                if bytes[j] == marker {
                    // Check it's not preceded or followed by the same marker (doubled)
                    let preceded_by_marker = j > 0 && bytes[j - 1] == marker;
                    let followed_by_marker = j + 1 < len && bytes[j + 1] == marker;

                    if !preceded_by_marker && !followed_by_marker {
                        // Found a valid closing marker
                        let full = &line[start..=j];
                        matches.push(EmphasisMatch {
                            full_match: full.to_string(),
                            style: if marker == b'*' {
                                "asterisk".to_string()
                            } else {
                                "underscore".to_string()
                            },
                            start,
                        });
                        i = j + 1;
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
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    matches
}

impl Rule for MD049 {
    fn names(&self) -> &[&'static str] {
        &["MD049", "emphasis-style"]
    }

    fn description(&self) -> &'static str {
        "Emphasis style should be consistent"
    }

    fn tags(&self) -> &[&'static str] {
        &["emphasis"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md049.md")
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

        // First pass: collect all emphasis occurrences to determine preferred style
        let mut all_matches: Vec<(usize, EmphasisMatch)> = Vec::new(); // (line_number, match)

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            for em in find_emphasis_matches(line) {
                all_matches.push((line_number, em));
            }
        }

        if all_matches.is_empty() {
            return errors;
        }

        // Determine the preferred style
        let preferred_style = if configured_style == "consistent" {
            // Use the style of the first occurrence
            all_matches[0].1.style.clone()
        } else {
            configured_style.clone()
        };

        // Second pass: report errors for wrong-style emphasis with fix_info
        for (line_number, em) in &all_matches {
            if em.style != preferred_style {
                let corrected = if preferred_style == "asterisk" {
                    // Replace _text_ with *text*
                    let inner = &em.full_match[1..em.full_match.len() - 1];
                    format!("*{}*", inner)
                } else {
                    // Replace *text* with _text_
                    let inner = &em.full_match[1..em.full_match.len() - 1];
                    format!("_{}_", inner)
                };

                errors.push(LintError {
                    line_number: *line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: Some(format!(
                        "Expected: {}; Actual: {}",
                        preferred_style, em.style
                    )),
                    error_context: Some(em.full_match.clone()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((em.start + 1, em.full_match.len())),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(em.start + 1), // 1-based
                        delete_count: Some(em.full_match.len() as i32),
                        insert_text: Some(corrected),
                    }),
                    severity: Severity::Error,
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
        lines: &'a [String],
        tokens: &'a [crate::parser::Token],
        config: &'a HashMap<String, serde_json::Value>,
    ) -> crate::types::RuleParams<'a> {
        crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens,
            config,
        }
    }

    #[test]
    fn test_md049_consistent_asterisks() {
        let rule = MD049;
        let lines: Vec<String> = vec!["*one* and *two* and *three*\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md049_consistent_underscores() {
        let rule = MD049;
        let lines: Vec<String> = vec!["_one_ and _two_ and _three_\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md049_mixed_styles_consistent_mode() {
        let rule = MD049;
        // First emphasis is asterisk, so underscore ones should be flagged
        let lines: Vec<String> = vec!["*one* and _two_\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].error_detail.as_deref(),
            Some("Expected: asterisk; Actual: underscore")
        );
    }

    #[test]
    fn test_md049_configured_asterisk_style() {
        let rule = MD049;
        let lines: Vec<String> = vec!["_one_ and _two_\n".to_string()];
        let tokens = vec![];
        let mut config = HashMap::new();
        config.insert("style".to_string(), serde_json::json!("asterisk"));
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md049_configured_underscore_style() {
        let rule = MD049;
        let lines: Vec<String> = vec!["*one* and *two*\n".to_string()];
        let tokens = vec![];
        let mut config = HashMap::new();
        config.insert("style".to_string(), serde_json::json!("underscore"));
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md049_fix_info_underscore_to_asterisk() {
        let rule = MD049;
        // First is asterisk, so underscore should get fix_info to convert to asterisk
        let lines: Vec<String> = vec!["*one* and _two_\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        assert_eq!(fix.line_number, None);
        // "_two_" starts at index 10, 1-based = 11
        assert_eq!(fix.edit_column, Some(11));
        assert_eq!(fix.delete_count, Some(5)); // "_two_".len() == 5
        assert_eq!(fix.insert_text, Some("*two*".to_string()));
    }

    #[test]
    fn test_md049_fix_info_asterisk_to_underscore() {
        let rule = MD049;
        // First is underscore, so asterisk should get fix_info to convert to underscore
        let lines: Vec<String> = vec!["_one_ and *two*\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        // "*two*" starts at index 10, 1-based = 11
        assert_eq!(fix.edit_column, Some(11));
        assert_eq!(fix.delete_count, Some(5));
        assert_eq!(fix.insert_text, Some("_two_".to_string()));
    }

    #[test]
    fn test_md049_fix_info_multiple_errors() {
        let rule = MD049;
        let lines: Vec<String> = vec!["*ok* and _bad1_ and _bad2_\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);

        let fix0 = errors[0].fix_info.as_ref().expect("first fix_info");
        assert_eq!(fix0.insert_text, Some("*bad1*".to_string()));

        let fix1 = errors[1].fix_info.as_ref().expect("second fix_info");
        assert_eq!(fix1.insert_text, Some("*bad2*".to_string()));
    }

    #[test]
    fn test_md049_no_emphasis() {
        let rule = MD049;
        let lines: Vec<String> = vec!["Just plain text.\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md049_does_not_match_strong() {
        let rule = MD049;
        // **bold** should NOT be treated as emphasis
        let lines: Vec<String> = vec!["**bold** and __also bold__\n".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
