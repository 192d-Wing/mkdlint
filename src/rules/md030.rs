//! MD030 - Spaces after list markers
//!
//! This rule checks for the number of spaces between a list marker (e.g. '-', '*', '+' or '1.')
//! and the text of the list item.

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD030;

impl Rule for MD030 {
    fn names(&self) -> &[&'static str] {
        &["MD030", "list-marker-space"]
    }

    fn description(&self) -> &'static str {
        "Spaces after list markers"
    }

    fn tags(&self) -> &[&'static str] {
        &["ol", "ul", "whitespace", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md030.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get configuration
        let ul_single = params
            .config
            .get("ul_single")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        let ol_single = params
            .config
            .get("ol_single")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        let ul_multi = params
            .config
            .get("ul_multi")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        let ol_multi = params
            .config
            .get("ol_multi")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        // Find all list tokens (ordered and unordered)
        let lists = params
            .tokens
            .filter_by_types(&["listOrdered", "listUnordered"]);

        for list in lists {
            let ordered = list.token_type == "listOrdered";

            // Get all listItemPrefix tokens that are children of this list
            let list_item_prefixes: Vec<_> = list
                .children
                .iter()
                .filter_map(|&child_idx| params.tokens.get(child_idx))
                .flat_map(|list_item| {
                    list_item
                        .children
                        .iter()
                        .filter_map(|&prefix_idx| params.tokens.get(prefix_idx))
                        .filter(|token| token.token_type == "listItemPrefix")
                })
                .collect();

            if list_item_prefixes.is_empty() {
                continue;
            }

            // Determine if all items are single-line
            let list_line_count = list.end_line - list.start_line + 1;
            let all_single_line = list_line_count == list_item_prefixes.len();

            // Choose expected spaces based on list type and single/multi-line
            let expected_spaces = if ordered {
                if all_single_line { ol_single } else { ol_multi }
            } else if all_single_line {
                ul_single
            } else {
                ul_multi
            };

            // Check each listItemPrefix for whitespace
            for list_item_prefix in list_item_prefixes {
                // Get the range for the entire list item prefix
                let range = (
                    list_item_prefix.start_column,
                    list_item_prefix.end_column - list_item_prefix.start_column,
                );

                // Find listItemPrefixWhitespace tokens within this prefix
                let whitespace_tokens: Vec<_> = list_item_prefix
                    .children
                    .iter()
                    .filter_map(|&ws_idx| params.tokens.get(ws_idx))
                    .filter(|token| token.token_type == "listItemPrefixWhitespace")
                    .collect();

                for whitespace in whitespace_tokens {
                    let actual_spaces = whitespace.end_column - whitespace.start_column;

                    if actual_spaces != expected_spaces {
                        let fix_info = FixInfo {
                            line_number: None,
                            edit_column: Some(whitespace.start_column),
                            delete_count: Some(actual_spaces as i32),
                            insert_text: Some(" ".repeat(expected_spaces)),
                        };

                        errors.push(LintError {
                            line_number: whitespace.start_line,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(format!(
                                "Expected: {}; Actual: {}",
                                expected_spaces, actual_spaces
                            )),
                            error_context: None,
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some(range),
                            fix_info: Some(fix_info),
                            suggestion: Some(
                                "Use consistent spacing after list marker".to_string(),
                            ),
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
    use crate::parser::Token;
    use std::collections::HashMap;

    fn create_list_token(
        token_type: &str,
        start_line: usize,
        end_line: usize,
        children: Vec<usize>,
    ) -> Token {
        Token {
            token_type: token_type.to_string(),
            start_line,
            start_column: 1,
            end_line,
            end_column: 1,
            text: String::new(),
            children,
            parent: None,
            metadata: HashMap::new(),
        }
    }

    fn create_list_item_token(
        start_line: usize,
        end_line: usize,
        children: Vec<usize>,
        parent: Option<usize>,
    ) -> Token {
        Token {
            token_type: "listItem".to_string(),
            start_line,
            start_column: 1,
            end_line,
            end_column: 1,
            text: String::new(),
            children,
            parent,
            metadata: HashMap::new(),
        }
    }

    fn create_list_item_prefix_token(
        line: usize,
        start_col: usize,
        end_col: usize,
        children: Vec<usize>,
        parent: Option<usize>,
    ) -> Token {
        Token {
            token_type: "listItemPrefix".to_string(),
            start_line: line,
            start_column: start_col,
            end_line: line,
            end_column: end_col,
            text: String::new(),
            children,
            parent,
            metadata: HashMap::new(),
        }
    }

    fn create_whitespace_token(
        line: usize,
        start_col: usize,
        end_col: usize,
        parent: Option<usize>,
    ) -> Token {
        Token {
            token_type: "listItemPrefixWhitespace".to_string(),
            start_line: line,
            start_column: start_col,
            end_line: line,
            end_column: end_col,
            text: String::new(),
            children: vec![],
            parent,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_md030_single_space_correct() {
        // - Item (1 space after marker)
        let tokens = vec![
            create_list_token("listUnordered", 1, 1, vec![1]), // 0: list
            create_list_item_token(1, 1, vec![2], Some(0)),    // 1: listItem
            create_list_item_prefix_token(1, 1, 3, vec![3], Some(1)), // 2: listItemPrefix "- "
            create_whitespace_token(1, 2, 3, Some(2)),         // 3: whitespace (1 space)
        ];

        let lines = vec!["- Item\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD030;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md030_two_spaces_violation() {
        // -  Item (2 spaces after marker, expected 1)
        let tokens = vec![
            create_list_token("listUnordered", 1, 1, vec![1]), // 0: list
            create_list_item_token(1, 1, vec![2], Some(0)),    // 1: listItem
            create_list_item_prefix_token(1, 1, 4, vec![3], Some(1)), // 2: listItemPrefix "-  "
            create_whitespace_token(1, 2, 4, Some(2)),         // 3: whitespace (2 spaces)
        ];

        let lines = vec!["-  Item\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD030;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Expected: 1; Actual: 2")
        );
    }

    #[test]
    fn test_md030_ordered_list_single_space() {
        // 1. Item (1 space after marker)
        let tokens = vec![
            create_list_token("listOrdered", 1, 1, vec![1]), // 0: list
            create_list_item_token(1, 1, vec![2], Some(0)),  // 1: listItem
            create_list_item_prefix_token(1, 1, 4, vec![3], Some(1)), // 2: listItemPrefix "1. "
            create_whitespace_token(1, 3, 4, Some(2)),       // 3: whitespace (1 space)
        ];

        let lines = vec!["1. Item\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD030;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md030_ordered_list_two_spaces_violation() {
        // 1.  Item (2 spaces after marker, expected 1)
        let tokens = vec![
            create_list_token("listOrdered", 1, 1, vec![1]), // 0: list
            create_list_item_token(1, 1, vec![2], Some(0)),  // 1: listItem
            create_list_item_prefix_token(1, 1, 5, vec![3], Some(1)), // 2: listItemPrefix "1.  "
            create_whitespace_token(1, 3, 5, Some(2)),       // 3: whitespace (2 spaces)
        ];

        let lines = vec!["1.  Item\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD030;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Expected: 1; Actual: 2")
        );
    }

    #[test]
    fn test_md030_multi_line_config() {
        // Multi-line list with ul_multi = 3
        let tokens = vec![
            create_list_token("listUnordered", 1, 3, vec![1, 4]), // 0: list
            create_list_item_token(1, 2, vec![2], Some(0)),       // 1: listItem
            create_list_item_prefix_token(1, 1, 5, vec![3], Some(1)), // 2: listItemPrefix "-   "
            create_whitespace_token(1, 2, 5, Some(2)),            // 3: whitespace (3 spaces)
            create_list_item_token(3, 3, vec![5], Some(0)),       // 4: listItem
            create_list_item_prefix_token(3, 1, 5, vec![6], Some(4)), // 5: listItemPrefix "-   "
            create_whitespace_token(3, 2, 5, Some(5)),            // 6: whitespace (3 spaces)
        ];

        let lines = vec![
            "-   Item 1\n".to_string(),
            "    Paragraph 2\n".to_string(),
            "-   Item 2\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert("ul_multi".to_string(), serde_json::json!(3));

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD030;
        let errors = rule.lint(&params);
        // Should not error since it's multi-line and we configured ul_multi to 3
        assert_eq!(errors.len(), 0);
    }
}
