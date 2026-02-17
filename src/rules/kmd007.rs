//! KMD007 - Math block delimiters must be matched
//!
//! In Kramdown, display math is fenced with `$$` on its own line:
//!
//! ```text
//! $$
//! \begin{aligned}
//!   x = 1
//! \end{aligned}
//! $$
//! ```
//!
//! This rule fires when an opening `$$` fence has no matching closing `$$`.

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct KMD007;

impl Rule for KMD007 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD007", "math-block-delimiters"]
    }

    fn description(&self) -> &'static str {
        "Math block '$$' delimiters must be matched"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "math"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn is_enabled_by_default(&self) -> bool {
        false
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let lines = params.lines;

        let mut in_code_block = false;
        let mut math_open_line: Option<usize> = None; // line number of opening $$

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r').trim();

            // Track code fences — math inside code blocks is not processed
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // A line that is exactly `$$` is a math block fence
            if trimmed == "$$" {
                if let Some(open_line) = math_open_line.take() {
                    // Closing fence — matched, nothing to report
                    let _ = open_line;
                } else {
                    // Opening fence
                    math_open_line = Some(idx + 1);
                }
            }
        }

        // If still open at EOF, report the unclosed block
        if let Some(open_line) = math_open_line {
            errors.push(LintError {
                line_number: open_line,
                rule_names: self.names(),
                rule_description: self.description(),
                error_detail: Some(format!(
                    "Unclosed math block: opening '$$' on line {open_line} has no matching closing '$$'"
                )),
                severity: Severity::Error,
                ..Default::default()
            });
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleParams;
    use std::collections::HashMap;

    fn lint(content: &str) -> Vec<LintError> {
        let lines: Vec<&str> = content.split_inclusive('\n').collect();
        let rule = KMD007;
        rule.lint(&RuleParams {
            name: "test.md",
            version: "0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        })
    }

    #[test]
    fn test_kmd007_matched_math_block_ok() {
        let errors = lint("# H\n\n$$\nx = 1\n$$\n");
        assert!(errors.is_empty(), "matched $$ should not fire");
    }

    #[test]
    fn test_kmd007_unclosed_math_block() {
        let errors = lint("# H\n\n$$\nx = 1\n");
        assert!(
            errors.iter().any(|e| e.rule_names[0] == "KMD007"),
            "should fire on unclosed math block"
        );
    }

    #[test]
    fn test_kmd007_multiple_math_blocks_ok() {
        let errors = lint("# H\n\n$$\na\n$$\n\nText\n\n$$\nb\n$$\n");
        assert!(errors.is_empty(), "two matched blocks should not fire");
    }

    #[test]
    fn test_kmd007_inside_code_block_ignored() {
        let errors = lint("# H\n\n```\n$$\nmath\n```\n");
        assert!(errors.is_empty(), "should not fire inside code blocks");
    }

    #[test]
    fn test_kmd007_no_math_ok() {
        let errors = lint("# H\n\nPlain paragraph.\n");
        assert!(errors.is_empty(), "no math should not fire");
    }
}
