//! MD013 - Line length
//!
//! This rule checks that lines are not longer than a configured limit

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD013;

impl Rule for MD013 {
    fn names(&self) -> &'static [&'static str] {
        &["MD013", "line-length"]
    }

    fn description(&self) -> &'static str {
        "Line length"
    }

    fn tags(&self) -> &[&'static str] {
        &["line_length"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md013.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let line_length = params
            .config
            .get("line_length")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as usize;
        let mut in_code_block = false;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Check for code block fences
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip code blocks, tables, and headings
            if in_code_block || trimmed.starts_with('|') || trimmed.starts_with('#') {
                continue;
            }

            let actual_length = trimmed.chars().count();
            if actual_length > line_length {
                errors.push(LintError {
                    line_number,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!(
                        "Expected: {}; Actual: {}",
                        line_length, actual_length
                    )),
                    error_context: Some(if actual_length > 78 {
                        let truncated: String = trimmed.chars().take(75).collect();
                        format!("{}...", truncated)
                    } else {
                        trimmed.to_string()
                    }),
                    rule_information: self.information(),
                    error_range: Some((line_length + 1, actual_length - line_length)),
                    fix_info: None,
                    suggestion: Some(
                        "Consider breaking long lines for better readability".to_string(),
                    ),
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

    #[test]
    fn test_md013_short_line() {
        let lines = vec!["Short line\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD013;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md013_long_line() {
        let long_line = "a".repeat(100) + "\n";
        let lines = vec![long_line.as_str()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD013;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md013_code_block_excluded() {
        let long_code = "a".repeat(120) + "\n";
        let lines = vec!["```\n", long_code.as_str(), "```\n"];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };
        let rule = MD013;
        let errors = rule.lint(&params);
        assert_eq!(
            errors.len(),
            0,
            "Long lines in code blocks should be excluded"
        );
    }

    #[test]
    fn test_md013_heading_excluded() {
        let long_heading = format!("# {}\n", "a".repeat(120));
        let lines = vec![long_heading.as_str()];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };
        let rule = MD013;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0, "Long headings should be excluded");
    }
}
