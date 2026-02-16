//! MD059 - Emphasis marker style in math

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

// Pattern to detect emphasis-style underscores: _text_ within math
static EMPHASIS_UNDERSCORE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"_[^_]+_").unwrap());

pub struct MD059;

impl Rule for MD059 {
    fn names(&self) -> &[&'static str] {
        &["MD059", "emphasis-marker-style-math"]
    }

    fn description(&self) -> &'static str {
        "Emphasis marker style should not conflict with math syntax"
    }

    fn tags(&self) -> &[&'static str] {
        &["emphasis", "math"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md059.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut in_display_math = false;
        let mut display_math_content = String::new();
        let mut display_math_start_line: usize = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Handle display math blocks ($$...$$)
            // Check for $$ delimiters on a line
            if trimmed.trim() == "$$" {
                if in_display_math {
                    // Closing $$: check accumulated content
                    if EMPHASIS_UNDERSCORE_RE.is_match(&display_math_content) {
                        errors.push(LintError {
                            line_number: display_math_start_line,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(
                                "Emphasis-style underscore found in display math".to_string(),
                            ),
                            error_context: Some(display_math_content.trim().to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: None,
                            fix_info: None,
                            suggestion: Some("Use backslash to escape $ for literal dollar signs when using math".to_string()),
                            severity: Severity::Warning,
                        });
                    }
                    in_display_math = false;
                    display_math_content.clear();
                } else {
                    // Opening $$
                    in_display_math = true;
                    display_math_start_line = line_number;
                }
                continue;
            }

            if in_display_math {
                display_math_content.push_str(trimmed);
                display_math_content.push(' ');
                continue;
            }

            // Handle inline display math ($$...$$) on a single line
            self.check_inline_display_math(trimmed, line_number, &mut errors);

            // Handle inline math ($...$) on a single line
            self.check_inline_math(trimmed, line_number, &mut errors);
        }

        errors
    }
}

impl MD059 {
    /// Check for emphasis underscores in single-line $$...$$ math
    fn check_inline_display_math(
        &self,
        line: &str,
        line_number: usize,
        errors: &mut Vec<LintError>,
    ) {
        // Find $$...$$ spans that are on a single line (not standalone $$)
        let mut search_start = 0;
        while let Some(start) = line[search_start..].find("$$") {
            let abs_start = search_start + start;
            let after_open = abs_start + 2;
            if after_open >= line.len() {
                break;
            }
            if let Some(end) = line[after_open..].find("$$") {
                let abs_end = after_open + end;
                let math_content = &line[after_open..abs_end];
                if EMPHASIS_UNDERSCORE_RE.is_match(math_content) {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(
                            "Emphasis-style underscore found in display math".to_string(),
                        ),
                        error_context: Some(format!("$${}$$", math_content)),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: Some((abs_start + 1, abs_end + 2 - abs_start)),
                        fix_info: None,
                        suggestion: Some(
                            "Use backslash to escape $ for literal dollar signs when using math"
                                .to_string(),
                        ),
                        severity: Severity::Warning,
                    });
                }
                search_start = abs_end + 2;
            } else {
                break;
            }
        }
    }

    /// Check for emphasis underscores in single-line $...$ math
    fn check_inline_math(&self, line: &str, line_number: usize, errors: &mut Vec<LintError>) {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            // Skip $$ (handled by display math)
            if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'$' {
                i += 2;
                // Skip past the closing $$
                while i + 1 < bytes.len() {
                    if bytes[i] == b'$' && bytes[i + 1] == b'$' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            if bytes[i] == b'$' {
                let start = i;
                i += 1;
                // Find the closing $
                while i < bytes.len() && bytes[i] != b'$' {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'$' {
                    let math_content = &line[start + 1..i];
                    if !math_content.is_empty() && EMPHASIS_UNDERSCORE_RE.is_match(math_content) {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(
                                "Emphasis-style underscore found in math".to_string(),
                            ),
                            error_context: Some(format!("${}$", math_content)),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some((start + 1, i + 1 - start)),
                            fix_info: None,
                            suggestion: Some("Use backslash to escape $ for literal dollar signs when using math".to_string()),
                            severity: Severity::Warning,
                        });
                    }
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
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
    fn test_md059_no_emphasis_in_math() {
        let lines = vec!["$x^2$\n".to_string()];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let rule = MD059;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md059_emphasis_in_math() {
        let lines = vec!["$_text_$\n".to_string()];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let rule = MD059;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert_eq!(errors[0].severity, Severity::Warning);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("underscore")
        );
    }
}
