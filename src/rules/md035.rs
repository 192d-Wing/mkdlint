//! MD035 - Horizontal rule style

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD035;

impl Rule for MD035 {
    fn names(&self) -> &[&'static str] {
        &["MD035", "hr-style"]
    }

    fn description(&self) -> &'static str {
        "Horizontal rule style"
    }

    fn tags(&self) -> &[&'static str] {
        &["hr"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md035.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get the style configuration, default to "consistent"
        let mut style = params
            .config
            .get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("consistent")
            .trim()
            .to_string();

        // Filter for thematic break tokens (horizontal rules)
        let thematic_breaks = params.tokens.filter_by_type("thematicBreak");

        for token in thematic_breaks {
            let line_number = token.start_line;
            let text = &token.text;

            // If style is "consistent", use the first horizontal rule as the style
            if style == "consistent" {
                style = text.clone();
            }

            // Check if the current horizontal rule matches the expected style
            if text != &style {
                errors.push(LintError {
                    line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: Some(format!("Expected: {}; Actual: {}", style, text)),
                    error_context: Some(text.clone()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((1, text.len())),
                    fix_info: Some(FixInfo {
                        line_number: Some(line_number),
                        edit_column: Some(1),
                        delete_count: Some(text.len() as i32),
                        insert_text: Some(style.clone()),
                    }),
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
    use crate::parser::Token;
    use std::collections::HashMap;

    #[test]
    fn test_md035_consistent_style() {
        let tokens = vec![
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 4,
                text: "---".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 3,
                start_column: 1,
                end_line: 3,
                end_column: 4,
                text: "---".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
        ];

        let lines = vec!["---\n".to_string(), "\n".to_string(), "---\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD035;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md035_inconsistent_style() {
        let tokens = vec![
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 4,
                text: "---".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 3,
                start_column: 1,
                end_line: 3,
                end_column: 4,
                text: "***".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
        ];

        let lines = vec!["---\n".to_string(), "\n".to_string(), "***\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD035;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
        assert_eq!(
            errors[0].error_detail,
            Some("Expected: ---; Actual: ***".to_string())
        );
    }

    #[test]
    fn test_md035_specific_style() {
        let tokens = vec![
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 4,
                text: "---".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 3,
                start_column: 1,
                end_line: 3,
                end_column: 4,
                text: "***".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
        ];

        let lines = vec!["---\n".to_string(), "\n".to_string(), "***\n".to_string()];

        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("***".to_string()),
        );

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD035;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert_eq!(
            errors[0].error_detail,
            Some("Expected: ***; Actual: ---".to_string())
        );
    }

    #[test]
    fn test_md035_multiple_inconsistencies() {
        let tokens = vec![
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 4,
                text: "---".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 3,
                start_column: 1,
                end_line: 3,
                end_column: 4,
                text: "***".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 5,
                start_column: 1,
                end_line: 5,
                end_column: 6,
                text: "* * *".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
        ];

        let lines = vec![
            "---\n".to_string(),
            "\n".to_string(),
            "***\n".to_string(),
            "\n".to_string(),
            "* * *\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD035;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 3);
        assert_eq!(errors[1].line_number, 5);
    }

    #[test]
    fn test_md035_no_horizontal_rules() {
        let tokens = vec![];
        let lines = vec!["# Heading\n".to_string(), "Some text\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD035;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md035_fix_info() {
        let tokens = vec![
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 4,
                text: "---".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
            Token {
                token_type: "thematicBreak".to_string(),
                start_line: 3,
                start_column: 1,
                end_line: 3,
                end_column: 4,
                text: "***".to_string(),
                children: vec![],
                parent: None,
                metadata: HashMap::new(),
            },
        ];

        let lines = vec!["---\n".to_string(), "\n".to_string(), "***\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD035;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("Should have fix_info");
        assert_eq!(fix.line_number, Some(3));
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(3));
        assert_eq!(fix.insert_text, Some("---".to_string()));
    }
}
