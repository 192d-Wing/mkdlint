//! MD048 - Code fence style

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD048;

impl Rule for MD048 {
    fn names(&self) -> &'static [&'static str] {
        &["MD048", "code-fence-style"]
    }

    fn description(&self) -> &'static str {
        "Code fence style"
    }

    fn tags(&self) -> &[&'static str] {
        &["code", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md048.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut backtick_lines = Vec::new();
        let mut tilde_lines = Vec::new();
        let mut first_style_line = 0;
        let mut first_style = "";

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                if first_style_line == 0 {
                    first_style_line = line_number;
                    first_style = "```";
                }
                backtick_lines.push(line_number);
            } else if trimmed.starts_with("~~~") {
                if first_style_line == 0 {
                    first_style_line = line_number;
                    first_style = "~~~";
                }
                tilde_lines.push(line_number);
            }
        }

        // If both styles are used, convert all to the first style encountered
        if !backtick_lines.is_empty() && !tilde_lines.is_empty() {
            // Report errors for each line that needs to be converted
            let lines_to_fix = if first_style == "```" {
                &tilde_lines
            } else {
                &backtick_lines
            };

            for &line_num in lines_to_fix {
                let line_idx = line_num - 1;
                let line = &params.lines[line_idx];
                let trimmed = line.trim();

                // Determine what to replace
                let (old_fence, new_fence) = if trimmed.starts_with("~~~") {
                    ("~~~", "```")
                } else {
                    ("```", "~~~")
                };

                // Find the fence prefix (could be `~~~rust` or `~~~`)
                let fence_len = old_fence.len();
                let leading_spaces = line.len() - line.trim_start().len();

                errors.push(LintError {
                    line_number: line_num,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!("Expected: {}; Actual: {}", first_style, old_fence)),
                    error_context: Some(trimmed.to_string()),
                    rule_information: self.information(),
                    error_range: Some((leading_spaces + 1, fence_len)),
                    fix_info: Some(FixInfo {
                        line_number: Some(line_num),
                        edit_column: Some(leading_spaces + 1),
                        delete_count: Some(fence_len as i32),
                        insert_text: Some(new_fence.to_string()),
                    }),
                    suggestion: Some("Use consistent code fence style".to_string()),
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

    fn make_params<'a>(
        lines: &'a [&'a str],
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
    fn test_md048_consistent_backticks() {
        let rule = MD048;
        let lines: Vec<&str> = vec![
            "```\n",
            "code block 1\n",
            "```\n",
            "\n",
            "```\n",
            "code block 2\n",
            "```\n",
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md048_consistent_tildes() {
        let rule = MD048;
        let lines: Vec<&str> = vec![
            "~~~\n",
            "code block 1\n",
            "~~~\n",
            "\n",
            "~~~\n",
            "code block 2\n",
            "~~~\n",
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md048_mixed_styles() {
        let rule = MD048;
        let lines: Vec<&str> = vec![
            "```\n",
            "code block 1\n",
            "```\n",
            "\n",
            "~~~\n",
            "code block 2\n",
            "~~~\n",
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        // Should report 2 errors - both tilde fences need to be converted to backticks
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 5);
        assert_eq!(errors[1].line_number, 7);
    }

    #[test]
    fn test_md048_fix_info() {
        let rule = MD048;
        let lines: Vec<&str> = vec![
            "```rust\n",
            "let x = 5;\n",
            "```\n",
            "\n",
            "~~~python\n",
            "y = 10\n",
            "~~~\n",
        ];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);

        // Check first tilde fence fix_info
        let fix1 = errors[0].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix1.line_number, Some(5));
        assert_eq!(fix1.edit_column, Some(1));
        assert_eq!(fix1.delete_count, Some(3));
        assert_eq!(fix1.insert_text, Some("```".to_string()));

        // Check second tilde fence fix_info
        let fix2 = errors[1].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix2.line_number, Some(7));
        assert_eq!(fix2.edit_column, Some(1));
        assert_eq!(fix2.delete_count, Some(3));
        assert_eq!(fix2.insert_text, Some("```".to_string()));
    }
}
