//! KMD011 - Inline math spans must be balanced
//!
//! In Kramdown, inline math is written as `$...$`.  A line that contains an
//! odd number of unescaped `$` characters (outside code spans and block-math
//! fences) indicates an unclosed inline-math span.
//!
//! Notes:
//! - Lines that are exactly `$$` (a block-math fence handled by KMD007) are
//!   skipped.
//! - `$` characters inside backtick code spans are ignored.
//! - Escaped `\$` is not counted.

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct KMD011;

impl Rule for KMD011 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD011", "inline-math-balanced"]
    }

    fn description(&self) -> &'static str {
        "Inline math spans must have balanced '$' delimiters"
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

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Track code fences
            let fence_trimmed = trimmed.trim();
            if fence_trimmed.starts_with("```") || fence_trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Skip block-math fence lines (exactly `$$`) — handled by KMD007
            if fence_trimmed == "$$" {
                continue;
            }

            let dollar_count = count_dollars(trimmed);
            if !dollar_count.is_multiple_of(2) {
                errors.push(LintError {
                    line_number: idx + 1,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!(
                        "Odd number of '$' delimiters ({dollar_count}) on line — inline math span is not closed"
                    )),
                    severity: Severity::Error,
                    fix_only: false,
                    ..Default::default()
                });
            }
        }

        errors
    }
}

/// Count unescaped `$` characters outside of backtick code spans.
fn count_dollars(line: &str) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut count = 0;
    let mut i = 0;

    while i < len {
        match chars[i] {
            // Escaped character — skip the next char
            '\\' => {
                i += 2;
            }
            // Backtick — skip to the matching closing backtick(s)
            '`' => {
                // Determine run length of opening backticks
                let start = i;
                while i < len && chars[i] == '`' {
                    i += 1;
                }
                let tick_run = i - start;
                // Find matching closing run
                'outer: while i < len {
                    if chars[i] == '`' {
                        let close_start = i;
                        while i < len && chars[i] == '`' {
                            i += 1;
                        }
                        if i - close_start == tick_run {
                            break 'outer;
                        }
                    } else {
                        i += 1;
                    }
                }
            }
            '$' => {
                count += 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleParams;
    use std::collections::HashMap;

    fn lint(content: &str) -> Vec<LintError> {
        let lines: Vec<&str> = content.split_inclusive('\n').collect();
        let rule = KMD011;
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
    fn test_kmd011_balanced_ok() {
        let errors = lint("# H\n\nSolve $x = 1$ and done.\n");
        assert!(errors.is_empty(), "balanced inline math should not fire");
    }

    #[test]
    fn test_kmd011_unbalanced() {
        let errors = lint("# H\n\nSolve $x = 1 and done.\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD011")),
            "odd number of $ should fire"
        );
    }

    #[test]
    fn test_kmd011_no_math_ok() {
        let errors = lint("# H\n\nPlain paragraph.\n");
        assert!(errors.is_empty(), "no $ should not fire");
    }

    #[test]
    fn test_kmd011_escaped_dollar_not_counted() {
        let errors = lint("# H\n\nPrice is \\$5 today.\n");
        assert!(
            errors.is_empty(),
            "escaped \\$ should not be counted as delimiter"
        );
    }

    #[test]
    fn test_kmd011_dollar_in_code_span_ignored() {
        let errors = lint("# H\n\nUse `$var` in shell.\n");
        assert!(
            errors.is_empty(),
            "$ inside backtick code span should be ignored"
        );
    }

    #[test]
    fn test_kmd011_dollar_in_code_block_ignored() {
        let errors = lint("# H\n\n```\n$var = 1\n```\n");
        assert!(errors.is_empty(), "$ inside code block should be ignored");
    }

    #[test]
    fn test_kmd011_block_math_fence_skipped() {
        // $$ on its own line is a block-math fence (KMD007), not inline math
        let errors = lint("# H\n\n$$\nx = 1\n$$\n");
        assert!(
            errors.is_empty(),
            "standalone $$ lines should not be counted"
        );
    }

    #[test]
    fn test_kmd011_double_dollar_inline_ok() {
        // $$ used inline (two $, even count)
        let errors = lint("# H\n\nThe expression $$x$$ is inline.\n");
        assert!(
            errors.is_empty(),
            "even number of $ in $$ inline should not fire"
        );
    }

    #[test]
    fn test_kmd011_multiple_spans_ok() {
        let errors = lint("# H\n\nBoth $a$ and $b$ are variables.\n");
        assert!(errors.is_empty(), "multiple balanced spans should not fire");
    }

    #[test]
    fn test_kmd011_error_line_number() {
        let errors = lint("# H\n\nLine with $unclosed.\n");
        assert_eq!(errors[0].line_number, 3, "error should point to line 3");
    }
}
