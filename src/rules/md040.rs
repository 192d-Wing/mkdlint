//! MD040 - Fenced code blocks should have a language specified

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD040;

impl Rule for MD040 {
    fn names(&self) -> &'static [&'static str] {
        &["MD040", "fenced-code-language"]
    }

    fn description(&self) -> &'static str {
        "Fenced code blocks should have a language specified"
    }

    fn tags(&self) -> &[&'static str] {
        &["code", "language", "fixable"]
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

            if crate::helpers::is_code_fence(trimmed) {
                let fence_chars = if trimmed.starts_with("```") {
                    "```"
                } else {
                    "~~~"
                };
                let after_fence = trimmed.trim_start_matches(fence_chars).trim();

                if in_code_block {
                    // This is a closing fence
                    in_code_block = false;
                } else {
                    // This is an opening fence - check if it has a language
                    in_code_block = true;
                    if after_fence.is_empty() {
                        // Get the configured default language (default: "text")
                        let default_lang = params
                            .config
                            .get("default_language")
                            .and_then(|v| v.as_str())
                            .unwrap_or("text");

                        let leading_spaces = line.len() - line.trim_start().len();
                        let fence_len = fence_chars.len();

                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some("Missing language specification".to_string()),
                            error_context: Some(trimmed.to_string()),
                            rule_information: self.information(),
                            error_range: Some((leading_spaces + 1, trimmed.len())),
                            fix_info: Some(FixInfo {
                                line_number: Some(line_number),
                                edit_column: Some(leading_spaces + fence_len + 1),
                                delete_count: None,
                                insert_text: Some(default_lang.to_string()),
                            }),
                            suggestion: Some(
                                "Specify a language for fenced code blocks".to_string(),
                            ),
                            severity: Severity::Error,
                            fix_only: false,
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
        let lines = vec!["```rust\n", "let x = 5;\n", "```\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD040;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md040_no_language() {
        let lines = vec!["```\n", "code\n", "```\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD040;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1); // Only the opening fence without language
        assert_eq!(errors[0].line_number, 1);
    }

    #[test]
    fn test_md040_fix_info() {
        let lines = vec!["```\n", "code here\n", "```\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD040;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix.line_number, Some(1));
        assert_eq!(fix.edit_column, Some(4)); // After ```
        assert_eq!(fix.delete_count, None);
        assert_eq!(fix.insert_text, Some("text".to_string()));
    }

    #[test]
    fn test_md040_custom_default_language() {
        let lines = vec!["~~~\n", "code here\n", "~~~\n"];

        let mut config = HashMap::new();
        config.insert(
            "default_language".to_string(),
            serde_json::Value::String("plaintext".to_string()),
        );

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &config,
            workspace_headings: None,
        };

        let rule = MD040;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix.insert_text, Some("plaintext".to_string()));
    }
}
