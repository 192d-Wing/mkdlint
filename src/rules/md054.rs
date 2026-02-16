//! MD054 - Link and image style

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

// Inline link: [text](url)
static INLINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]*)\]\(([^)]*)\)").unwrap());

// Full reference link: [text][label]
static FULL_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]*)\]\[([^\]]+)\]").unwrap());

// Collapsed reference link: [text][]
static COLLAPSED_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\[\]").unwrap());

// Shortcut reference link: [text] not followed by ( or [
static SHORTCUT_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\](?:[^(\[])").unwrap());

// Autolink: <http://...> or <https://...>
static AUTOLINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(https?://[^>]+)>").unwrap());

// Inline code span regex for stripping
static INLINE_CODE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`[^`]+`").unwrap());

// Code fence opening/closing
static CODE_FENCE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(`{3,}|~{3,})").unwrap());

pub struct MD054;

impl Rule for MD054 {
    fn names(&self) -> &[&'static str] {
        &["MD054", "link-image-style"]
    }

    fn description(&self) -> &'static str {
        "Link and image style"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "images"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md054.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Read config for allowed styles (all default to true)
        let allow_autolink = params
            .config
            .get("autolink")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_inline = params
            .config
            .get("inline")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_full = params
            .config
            .get("full")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_collapsed = params
            .config
            .get("collapsed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_shortcut = params
            .config
            .get("shortcut")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut in_code_block = false;
        let mut fence_pattern: Option<String> = None;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Track code fences
            if let Some(caps) = CODE_FENCE_RE.captures(trimmed) {
                let fence = caps.get(1).unwrap().as_str();
                if in_code_block {
                    // Check if the closing fence matches the opening fence type and length
                    if let Some(ref open) = fence_pattern {
                        let open_char = open.chars().next().unwrap();
                        let fence_char = fence.chars().next().unwrap();
                        if open_char == fence_char && fence.len() >= open.len() {
                            in_code_block = false;
                            fence_pattern = None;
                        }
                    }
                } else {
                    in_code_block = true;
                    fence_pattern = Some(fence.to_string());
                }
                continue;
            }

            if in_code_block {
                continue;
            }

            // Strip inline code spans to avoid false positives
            let processed = INLINE_CODE_RE.replace_all(trimmed, |caps: &regex::Captures| {
                " ".repeat(caps.get(0).unwrap().len())
            });

            // Check for each link style and report if disallowed.
            // Order matters: check more specific patterns first to avoid
            // a full reference being caught as inline, etc.

            // Track positions already matched to avoid double-reporting
            let mut matched_ranges: Vec<(usize, usize)> = Vec::new();

            // Autolink: <https://...>
            if !allow_autolink {
                for mat in AUTOLINK_RE.find_iter(&processed) {
                    matched_ranges.push((mat.start(), mat.end()));
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some("Autolink style is not allowed".to_string()),
                        error_context: Some(mat.as_str().to_string()),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: Some((mat.start() + 1, mat.len())),
                        fix_info: None,
                        severity: Severity::Error,
                    });
                }
            } else {
                for mat in AUTOLINK_RE.find_iter(&processed) {
                    matched_ranges.push((mat.start(), mat.end()));
                }
            }

            // Inline: [text](url)
            if !allow_inline {
                for mat in INLINE_RE.find_iter(&processed) {
                    if !overlaps(&matched_ranges, mat.start(), mat.end()) {
                        matched_ranges.push((mat.start(), mat.end()));
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some("Inline style is not allowed".to_string()),
                            error_context: Some(mat.as_str().to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some((mat.start() + 1, mat.len())),
                            fix_info: None,
                            severity: Severity::Error,
                        });
                    }
                }
            } else {
                for mat in INLINE_RE.find_iter(&processed) {
                    if !overlaps(&matched_ranges, mat.start(), mat.end()) {
                        matched_ranges.push((mat.start(), mat.end()));
                    }
                }
            }

            // Collapsed reference: [text][]
            if !allow_collapsed {
                for mat in COLLAPSED_REF_RE.find_iter(&processed) {
                    if !overlaps(&matched_ranges, mat.start(), mat.end()) {
                        matched_ranges.push((mat.start(), mat.end()));
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(
                                "Collapsed reference style is not allowed".to_string(),
                            ),
                            error_context: Some(mat.as_str().to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some((mat.start() + 1, mat.len())),
                            fix_info: None,
                            severity: Severity::Error,
                        });
                    }
                }
            } else {
                for mat in COLLAPSED_REF_RE.find_iter(&processed) {
                    if !overlaps(&matched_ranges, mat.start(), mat.end()) {
                        matched_ranges.push((mat.start(), mat.end()));
                    }
                }
            }

            // Full reference: [text][label]
            if !allow_full {
                for mat in FULL_REF_RE.find_iter(&processed) {
                    if !overlaps(&matched_ranges, mat.start(), mat.end()) {
                        matched_ranges.push((mat.start(), mat.end()));
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some("Full reference style is not allowed".to_string()),
                            error_context: Some(mat.as_str().to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some((mat.start() + 1, mat.len())),
                            fix_info: None,
                            severity: Severity::Error,
                        });
                    }
                }
            } else {
                for mat in FULL_REF_RE.find_iter(&processed) {
                    if !overlaps(&matched_ranges, mat.start(), mat.end()) {
                        matched_ranges.push((mat.start(), mat.end()));
                    }
                }
            }

            // Shortcut reference: [text] not followed by ( or [
            if !allow_shortcut {
                for mat in SHORTCUT_REF_RE.find_iter(&processed) {
                    if !overlaps(&matched_ranges, mat.start(), mat.end()) {
                        matched_ranges.push((mat.start(), mat.end()));
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(
                                "Shortcut reference style is not allowed".to_string(),
                            ),
                            error_context: Some(mat.as_str().to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some((mat.start() + 1, mat.len())),
                            fix_info: None,
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        errors
    }
}

/// Check if a range overlaps with any already-matched range
fn overlaps(ranges: &[(usize, usize)], start: usize, end: usize) -> bool {
    ranges.iter().any(|&(s, e)| start < e && end > s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_params<'a>(
        lines: &'a [String],
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
    fn test_md054_all_styles_allowed() {
        let lines = vec![
            "[inline link](https://example.com)\n".to_string(),
            "[full ref][label]\n".to_string(),
            "[collapsed ref][]\n".to_string(),
            "[shortcut ref] is here\n".to_string(),
            "<https://example.com>\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let rule = MD054;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md054_inline_only() {
        let lines = vec!["[full ref][label]\n".to_string()];
        let mut config = HashMap::new();
        config.insert("full".to_string(), serde_json::Value::Bool(false));
        let params = make_params(&lines, &config);
        let rule = MD054;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Full reference")
        );
    }

    #[test]
    fn test_md054_autolink_disabled() {
        let lines = vec!["<https://example.com>\n".to_string()];
        let mut config = HashMap::new();
        config.insert("autolink".to_string(), serde_json::Value::Bool(false));
        let params = make_params(&lines, &config);
        let rule = MD054;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Autolink")
        );
    }
}
