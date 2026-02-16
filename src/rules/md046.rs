//! MD046 - Code block style

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD046;

impl Rule for MD046 {
    fn names(&self) -> &[&'static str] {
        &["MD046", "code-block-style"]
    }

    fn description(&self) -> &'static str {
        "Code block style"
    }

    fn tags(&self) -> &[&'static str] {
        &["code"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md046.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut fenced_count = 0;
        let mut indented_count = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let _line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                fenced_count += 1;
            } else if line.starts_with("    ") && !trimmed.is_empty() {
                // Potential indented code block
                indented_count += 1;
            }
        }

        // If both styles are used, report error
        if fenced_count > 0 && indented_count > 0 {
            errors.push(LintError {
                line_number: 1,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: Some("Mixed code block styles (fenced and indented)".to_string()),
                error_context: None,
                rule_information: self.information().map(|s| s.to_string()),
                error_range: None,
                fix_info: None,
                suggestion: Some(
                    "Use consistent code block style (fenced or indented)".to_string(),
                ),
                severity: Severity::Error,
            });
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_md046_fenced_only() {
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
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

        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 0, "Fenced-only should not trigger MD046");
    }

    #[test]
    fn test_md046_indented_only() {
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "    code block\n".to_string(),
            "    more code\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 0, "Indented-only should not trigger MD046");
    }

    #[test]
    fn test_md046_mixed_styles() {
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "```\n".to_string(),
            "fenced code\n".to_string(),
            "```\n".to_string(),
            "\n".to_string(),
            "    indented code\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert_eq!(
            errors[0].error_detail,
            Some("Mixed code block styles (fenced and indented)".to_string())
        );
    }

    #[test]
    fn test_md046_tilde_fenced() {
        let lines = vec![
            "~~~\n".to_string(),
            "code\n".to_string(),
            "~~~\n".to_string(),
            "\n".to_string(),
            "    indented\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let errors = MD046.lint(&params);
        assert_eq!(
            errors.len(),
            1,
            "Tilde fenced + indented should trigger mixed style error"
        );
    }

    #[test]
    fn test_md046_no_code_blocks() {
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "Just a paragraph.\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 0, "No code blocks should not trigger MD046");
    }

    #[test]
    fn test_md046_no_fix_info() {
        let lines = vec![
            "```\n".to_string(),
            "code\n".to_string(),
            "```\n".to_string(),
            "    indented\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].fix_info.is_none(),
            "MD046 should not have fix_info"
        );
    }
}
