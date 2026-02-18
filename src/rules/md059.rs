//! MD059 - Emphasis marker style in math

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

// Pattern to detect emphasis-style underscores: _text_ within math
// Uses a non-backslash char (or start) before the opening _, and the closing _
// must also not be preceded by a backslash.
static EMPHASIS_UNDERSCORE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|[^\\])(_[^_\\]+_)").unwrap());

pub struct MD059;

impl Rule for MD059 {
    fn names(&self) -> &'static [&'static str] {
        &["MD059", "emphasis-marker-style-math"]
    }

    fn description(&self) -> &'static str {
        "Emphasis marker style should not conflict with math syntax"
    }

    fn tags(&self) -> &[&'static str] {
        &["emphasis", "math", "fixable"]
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

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Handle display math blocks ($$...$$)
            // Check for $$ delimiters on a line
            if trimmed.trim() == "$$" {
                in_display_math = !in_display_math;
                continue;
            }

            if in_display_math {
                // Check this content line for emphasis underscores
                self.check_line_for_emphasis(trimmed, line_number, 0, "display math", &mut errors);
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
    /// Check a string for emphasis underscores and emit errors with fix_info.
    /// `base_offset` is the 0-based offset within the original line where `content` starts.
    fn check_line_for_emphasis(
        &self,
        content: &str,
        line_number: usize,
        base_offset: usize,
        math_type: &str,
        errors: &mut Vec<LintError>,
    ) {
        for caps in EMPHASIS_UNDERSCORE_RE.captures_iter(content) {
            let em_match = caps.get(1).unwrap();
            let matched_text = em_match.as_str();
            // Escape the underscores: _text_ -> \_text\_
            let inner = &matched_text[1..matched_text.len() - 1];
            let escaped = format!("\\_{}\\_", inner);
            let abs_col = base_offset + em_match.start();

            errors.push(LintError {
                line_number,
                rule_names: self.names(),
                rule_description: self.description(),
                error_detail: Some(format!("Emphasis-style underscore found in {}", math_type)),
                error_context: Some(matched_text.to_string()),
                rule_information: self.information(),
                error_range: Some((abs_col + 1, matched_text.len())),
                fix_info: Some(FixInfo {
                    line_number: None,
                    edit_column: Some(abs_col + 1),
                    delete_count: Some(matched_text.len() as i32),
                    insert_text: Some(escaped),
                }),
                suggestion: Some("Escape underscores with backslash in math context".to_string()),
                severity: Severity::Warning,
                fix_only: false,
            });
        }
    }

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
                self.check_line_for_emphasis(
                    math_content,
                    line_number,
                    after_open,
                    "display math",
                    errors,
                );
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
                    if !math_content.is_empty() {
                        self.check_line_for_emphasis(
                            math_content,
                            line_number,
                            start + 1,
                            "math",
                            errors,
                        );
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
    fn test_md059_no_emphasis_in_math() {
        let lines = vec!["$x^2$\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let rule = MD059;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md059_emphasis_in_math() {
        let lines = vec!["$_text_$\n"];
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

    #[test]
    fn test_md059_fix_info_inline_math() {
        // "$_text_$" — underscore match at column 2 (1-based), length 6
        let lines = vec!["$_text_$"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD059.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix.edit_column, Some(2)); // 1-based: after the $
        assert_eq!(fix.delete_count, Some(6)); // "_text_" is 6 chars
        assert_eq!(fix.insert_text, Some("\\_text\\_".to_string()));
    }

    #[test]
    fn test_md059_fix_info_display_math_block() {
        // The fix should target the content line (line 2), not the $$ start line
        let lines = vec!["$$\n", "_text_\n", "$$\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD059.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2, "Error should be on content line");
        let fix = errors[0].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(6));
        assert_eq!(fix.insert_text, Some("\\_text\\_".to_string()));
    }

    #[test]
    fn test_md059_fix_info_inline_display_math() {
        // "$$_x_$$" — underscore match inside $$...$$
        let lines = vec!["$$_x_$$"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD059.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix.edit_column, Some(3)); // after $$
        assert_eq!(fix.delete_count, Some(3)); // "_x_"
        assert_eq!(fix.insert_text, Some("\\_x\\_".to_string()));
    }

    #[test]
    fn test_md059_multiple_underscores() {
        // Two emphasis patterns in one math span
        let lines = vec!["$_a_ + _b_$"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD059.lint(&params);
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0].fix_info.as_ref().unwrap().insert_text,
            Some("\\_a\\_".to_string())
        );
        assert_eq!(
            errors[1].fix_info.as_ref().unwrap().insert_text,
            Some("\\_b\\_".to_string())
        );
    }

    #[test]
    fn test_md059_subscript_no_trigger() {
        // Single underscore (subscript like x_1) should not trigger — needs _text_ pattern
        let lines = vec!["$x_1$"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD059.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md059_indented_dollar_dollar() {
        // Indented $$ should still toggle display math
        let lines = vec!["  $$\n", "  _text_\n", "  $$\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD059.lint(&params);
        // The $$ lines have extra whitespace so `trimmed.trim() == "$$"` should match
        assert_eq!(
            errors.len(),
            1,
            "Indented $$ should toggle display math block"
        );
    }
}
