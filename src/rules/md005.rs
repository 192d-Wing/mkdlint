//! MD005 - Inconsistent indentation for list items at the same level
//!
//! This rule checks for inconsistent indentation for list items at the same level.
//! For unordered lists, all items at the same level must start at the same column.
//! For ordered lists, either all items must start at the same column, or all items
//! must have their markers right-aligned (end at the same column).
//!
//! Note: Auto-fix is only supported for ordered lists. For unordered lists,
//! use MD007 (ul-indent) which handles indentation correction more precisely.

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD005;

impl Rule for MD005 {
    fn names(&self) -> &'static [&'static str] {
        &["MD005", "list-indent"]
    }

    fn description(&self) -> &'static str {
        "Inconsistent indentation for list items at the same level"
    }

    fn tags(&self) -> &[&'static str] {
        &["bullet", "ul", "indentation"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md005.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get all list tokens (both ordered and unordered)
        let lists = params
            .tokens
            .filter_by_types(&["listOrdered", "listUnordered"]);

        for list in lists {
            let expected_indent = list.start_column - 1;
            let mut expected_end = 0;
            let mut end_matching = false;

            // Get all listItemPrefix children of this list
            let list_item_prefixes: Vec<_> = params
                .tokens
                .get_children(list)
                .into_iter()
                .filter(|token| token.token_type == "listItemPrefix")
                .collect();

            for list_item_prefix in list_item_prefixes {
                let line_number = list_item_prefix.start_line;
                let actual_indent = list_item_prefix.start_column - 1;
                let range = (1, list_item_prefix.end_column - 1);

                if list.token_type == "listUnordered" {
                    // For unordered lists, check if indent matches expected
                    if expected_indent != actual_indent {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!(
                                "Expected: {}; Actual: {}",
                                expected_indent, actual_indent
                            )),
                            error_context: None,
                            rule_information: self.information(),
                            error_range: Some(range),
                            fix_info: None, // No fixInfo; MD007 handles this scenario better
                            suggestion: Some(
                                "Match list item indentation to previous items".to_string(),
                            ),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }
                } else {
                    // For ordered lists, check for consistent indentation or right-aligned markers
                    let marker_length = list_item_prefix.text.trim().len();
                    let actual_end = list_item_prefix.start_column + marker_length - 1;

                    // Set expected_end from first item if not set
                    if expected_end == 0 {
                        expected_end = actual_end;
                    }

                    if (expected_indent != actual_indent) || end_matching {
                        if expected_end == actual_end {
                            // Markers are right-aligned, switch to end-matching mode
                            end_matching = true;
                        } else {
                            // Generate appropriate error message
                            let (detail, expected, actual) = if end_matching {
                                (
                                    format!(
                                        "Expected: ({}); Actual: ({})",
                                        expected_end, actual_end
                                    ),
                                    expected_end - marker_length,
                                    actual_end - marker_length,
                                )
                            } else {
                                (
                                    format!(
                                        "Expected: {}; Actual: {}",
                                        expected_indent, actual_indent
                                    ),
                                    expected_indent,
                                    actual_indent,
                                )
                            };

                            errors.push(LintError {
                                line_number,
                                rule_names: self.names(),
                                rule_description: self.description(),
                                error_detail: Some(detail),
                                error_context: None,
                                rule_information: self.information(),
                                error_range: Some(range),
                                fix_info: Some(FixInfo {
                                    line_number: None,
                                    edit_column: Some(expected.min(actual) + 1),
                                    delete_count: Some((actual as i32 - expected as i32).max(0)),
                                    insert_text: if expected > actual {
                                        Some(" ".repeat(expected - actual))
                                    } else {
                                        None
                                    },
                                }),
                                suggestion: Some(
                                    "Match list item indentation to previous items".to_string(),
                                ),
                                severity: Severity::Error,
                                fix_only: false,
                            });
                        }
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
    use crate::parser::Token;
    use std::collections::HashMap;

    fn create_list_token(
        token_type: &str,
        start_line: usize,
        start_column: usize,
        children: Vec<usize>,
    ) -> Token {
        Token {
            token_type: token_type.to_string(),
            start_line,
            start_column,
            end_line: start_line,
            end_column: start_column + 10,
            text: String::new(),
            children,
            parent: None,
            metadata: HashMap::new(),
        }
    }

    fn create_list_item_prefix(
        start_line: usize,
        start_column: usize,
        end_column: usize,
        text: &str,
        parent: usize,
    ) -> Token {
        Token {
            token_type: "listItemPrefix".to_string(),
            start_line,
            start_column,
            end_line: start_line,
            end_column,
            text: text.to_string(),
            children: vec![],
            parent: Some(parent),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_md005_unordered_list_consistent() {
        let tokens = vec![
            create_list_token("listUnordered", 1, 1, vec![1, 2, 3]),
            create_list_item_prefix(1, 1, 3, "- ", 0),
            create_list_item_prefix(2, 1, 3, "- ", 0),
            create_list_item_prefix(3, 1, 3, "- ", 0),
        ];

        let lines = vec!["- Item 1\n", "- Item 2\n", "- Item 3\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD005;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md005_unordered_list_inconsistent() {
        let tokens = vec![
            create_list_token("listUnordered", 1, 1, vec![1, 2, 3]),
            create_list_item_prefix(1, 1, 3, "- ", 0),
            create_list_item_prefix(2, 2, 4, "- ", 0), // Indented incorrectly
            create_list_item_prefix(3, 1, 3, "- ", 0),
        ];

        let lines = vec![
            "- Item 1\n",
            " - Item 2\n", // Extra space
            "- Item 3\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD005;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
        assert!(errors[0].error_detail.is_some());
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Expected: 0; Actual: 1")
        );
    }

    #[test]
    fn test_md005_ordered_list_consistent() {
        let tokens = vec![
            create_list_token("listOrdered", 1, 1, vec![1, 2, 3]),
            create_list_item_prefix(1, 1, 4, "1. ", 0),
            create_list_item_prefix(2, 1, 4, "2. ", 0),
            create_list_item_prefix(3, 1, 4, "3. ", 0),
        ];

        let lines = vec!["1. Item 1\n", "2. Item 2\n", "3. Item 3\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD005;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md005_ordered_list_right_aligned() {
        let tokens = vec![
            create_list_token("listOrdered", 1, 2, vec![1, 2, 3, 4]),
            create_list_item_prefix(1, 2, 5, " 1. ", 0),
            create_list_item_prefix(2, 2, 5, " 2. ", 0),
            create_list_item_prefix(3, 2, 5, " 9. ", 0),
            create_list_item_prefix(4, 1, 5, "10. ", 0), // Right-aligned with above
        ];

        let lines = vec![
            " 1. Item 1\n",
            " 2. Item 2\n",
            " 9. Item 9\n",
            "10. Item 10\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD005;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md005_ordered_list_inconsistent() {
        let tokens = vec![
            create_list_token("listOrdered", 1, 3, vec![1, 2, 3]),
            create_list_item_prefix(1, 3, 6, "1. ", 0),
            create_list_item_prefix(2, 2, 5, "2. ", 0), // Wrong indent
            create_list_item_prefix(3, 3, 6, "3. ", 0),
        ];

        let lines = vec![
            "  1. Item 1\n",
            " 2. Item 2\n", // Less indented
            "  3. Item 3\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD005;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
        assert!(errors[0].fix_info.is_some());
    }

    #[test]
    fn test_md005_empty_list() {
        let tokens = vec![create_list_token("listUnordered", 1, 1, vec![])];

        let lines = vec![""];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD005;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md005_ordered_list_with_fix_info() {
        let tokens = vec![
            create_list_token("listOrdered", 1, 3, vec![1, 2]),
            create_list_item_prefix(1, 3, 6, "1. ", 0),
            create_list_item_prefix(2, 2, 5, "2. ", 0), // One space less
        ];

        let lines = vec!["  1. Item 1\n", " 2. Item 2\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD005;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix_info = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix_info.edit_column, Some(2)); // Min of actual and expected + 1
        assert_eq!(fix_info.delete_count, Some(0));
        assert_eq!(fix_info.insert_text, Some(" ".to_string())); // Insert one space
    }
}
