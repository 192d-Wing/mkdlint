//! MD040 - Fenced code blocks should have a language specified

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD040;

impl Rule for MD040 {
    fn names(&self) -> &[&'static str] {
        &["MD040", "fenced-code-language"]
    }

    fn description(&self) -> &'static str {
        "Fenced code blocks should have a language specified"
    }

    fn tags(&self) -> &[&'static str] {
        &["code", "language"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md040.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut in_code_block = false;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let fence_chars = if trimmed.starts_with("```") { "```" } else { "~~~" };
                let after_fence = trimmed.trim_start_matches(fence_chars).trim();

                if in_code_block {
                    // This is a closing fence
                    in_code_block = false;
                } else {
                    // This is an opening fence - check if it has a language
                    in_code_block = true;
                    if after_fence.is_empty() {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: None,
                            error_context: Some(trimmed.to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some((1, trimmed.len())),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_md040_with_language() {
        let lines = vec![
            "```rust\n".to_string(),
            "let x = 5;\n".to_string(),
            "```\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD040;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md040_no_language() {
        let lines = vec![
            "```\n".to_string(),
            "code\n".to_string(),
            "```\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD040;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1); // Only the opening fence without language
        assert_eq!(errors[0].line_number, 1);
    }
}
