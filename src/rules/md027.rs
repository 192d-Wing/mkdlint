//! MD027 - Multiple spaces after blockquote symbol

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD027;

impl Rule for MD027 {
    fn names(&self) -> &[&'static str] {
        &["MD027", "no-multiple-space-blockquote"]
    }

    fn description(&self) -> &'static str {
        "Multiple spaces after blockquote symbol"
    }

    fn tags(&self) -> &[&'static str] {
        &["blockquote", "whitespace", "indentation", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md027.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_start();

            if let Some(after_bracket) = trimmed.strip_prefix('>') {
                let space_count = after_bracket.chars().take_while(|&c| c == ' ').count();

                if space_count > 1 {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!("Expected: 1; Actual: {}", space_count)),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: Some((2, space_count)),
                        fix_info: Some(FixInfo {
                            line_number: None,
                            edit_column: Some(2),
                            delete_count: Some((space_count - 1) as i32),
                            insert_text: None,
                        }),
                        suggestion: Some(
                            "Remove multiple spaces after blockquote symbol".to_string(),
                        ),
                        severity: Severity::Error,
                    });
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
    fn test_md027_single_space() {
        let lines = vec!["> Blockquote\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD027;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md027_multiple_spaces() {
        let lines = vec![">  Blockquote\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD027;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
