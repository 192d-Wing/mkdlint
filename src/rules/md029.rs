//! MD029 - Ordered list item prefix
//!
//! This rule checks that ordered list item prefixes are consistent.
//! Supported styles:
//! - `one`: All items should be prefixed with `1.` (1/1/1)
//! - `ordered`: Items should increment sequentially (1/2/3)
//! - `zero`: All items should be prefixed with `0.` (0/0/0)
//! - `consistent`: Auto-detect from first two items (default)

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD029;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListStyle {
    One,
    Ordered,
    Zero,
    Consistent,
}

impl ListStyle {
    fn from_str(s: &str) -> Self {
        match s {
            "one" => ListStyle::One,
            "ordered" => ListStyle::Ordered,
            "zero" => ListStyle::Zero,
            _ => ListStyle::Consistent,
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            ListStyle::One => "1/1/1",
            ListStyle::Ordered => "1/2/3",
            ListStyle::Zero => "0/0/0",
            ListStyle::Consistent => "consistent",
        }
    }
}

/// Extract the ordered list item number from a line
fn get_ordered_list_value(line: &str) -> Option<(usize, usize, usize)> {
    let trimmed = line.trim_start();

    // Find the first digit
    let mut num_str = String::new();
    let mut chars = trimmed.chars().peekable();

    while let Some(ch) = chars.peek() {
        if ch.is_ascii_digit() {
            num_str.push(*ch);
            chars.next();
        } else {
            break;
        }
    }

    // Check if followed by a period and whitespace or end of line
    if !num_str.is_empty()
        && let Some('.') = chars.next()
    {
        // Valid ordered list marker
        if let Ok(value) = num_str.parse::<usize>() {
            // Calculate column (1-based)
            let indent = line.len() - trimmed.len();
            let column = indent + 1;
            return Some((value, column, num_str.len()));
        }
    }

    None
}

/// Check if a token is an ordered list by examining its first list item
fn is_ordered_list(
    tokens: &[crate::parser::Token],
    lines: &[String],
    list_token: &crate::parser::Token,
) -> bool {
    // Check if any child list items are ordered
    for &child_idx in &list_token.children {
        if let Some(child) = tokens.get(child_idx)
            && child.token_type == "listItem"
            && child.start_line > 0
            && child.start_line <= lines.len()
        {
            let line = &lines[child.start_line - 1];
            return get_ordered_list_value(line).is_some();
        }
    }
    false
}

impl Rule for MD029 {
    fn names(&self) -> &[&'static str] {
        &["MD029", "ol-prefix"]
    }

    fn description(&self) -> &'static str {
        "Ordered list item prefix"
    }

    fn tags(&self) -> &[&'static str] {
        &["ol", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md029.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get style from config
        let style = params
            .config
            .get("style")
            .and_then(|v| v.as_str())
            .map(ListStyle::from_str)
            .unwrap_or(ListStyle::Consistent);

        // Find all ordered lists
        let lists = params.tokens.filter_by_type("list");

        for list in lists {
            // Only process ordered lists
            if !is_ordered_list(params.tokens, params.lines, list) {
                continue;
            }

            // Get all list items for this ordered list
            let mut list_items = Vec::new();
            for &child_idx in &list.children {
                if let Some(child) = params.tokens.get(child_idx)
                    && child.token_type == "listItem"
                {
                    list_items.push(child);
                }
            }

            if list_items.is_empty() {
                continue;
            }

            // Determine the effective style for this list
            let mut expected = 1;
            let mut incrementing = false;

            // Check for incrementing number pattern 1/2/3 or 0/1/2
            if list_items.len() >= 2
                && list_items[0].start_line > 0
                && list_items[0].start_line <= params.lines.len()
                && list_items[1].start_line > 0
                && list_items[1].start_line <= params.lines.len()
            {
                let first_line = &params.lines[list_items[0].start_line - 1];
                let second_line = &params.lines[list_items[1].start_line - 1];

                if let (Some((first_val, _, _)), Some((second_val, _, _))) = (
                    get_ordered_list_value(first_line),
                    get_ordered_list_value(second_line),
                ) && (second_val != 1 || first_val == 0)
                {
                    incrementing = true;
                    if first_val == 0 {
                        expected = 0;
                    }
                }
            }

            // Determine effective style
            let list_style = match style {
                ListStyle::One | ListStyle::Ordered | ListStyle::Zero => style,
                ListStyle::Consistent => {
                    if incrementing {
                        ListStyle::Ordered
                    } else {
                        ListStyle::One
                    }
                }
            };

            // Set initial expected value based on style
            if list_style == ListStyle::Zero {
                expected = 0;
            } else if list_style == ListStyle::One {
                expected = 1;
            }

            // Validate each list item marker
            for item in list_items {
                if item.start_line == 0 || item.start_line > params.lines.len() {
                    continue;
                }

                let line = &params.lines[item.start_line - 1];

                if let Some((actual, column, num_len)) = get_ordered_list_value(line) {
                    if actual != expected {
                        errors.push(LintError {
                            line_number: item.start_line,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(format!(
                                "Expected: {}; Actual: {}",
                                expected, actual
                            )),
                            error_context: Some(format!("Style: {}", list_style.to_str())),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: Some((column, num_len)),
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(column),
                                delete_count: Some(num_len as i32),
                                insert_text: Some(expected.to_string()),
                            }),
                            suggestion: Some("Use consistent list numbering style".to_string()),
                            severity: Severity::Error,
                        });
                    }

                    // Increment for ordered style
                    if list_style == ListStyle::Ordered {
                        expected += 1;
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

    fn create_list_item_token(line: usize, parent: Option<usize>) -> Token {
        Token {
            token_type: "listItem".to_string(),
            start_line: line,
            start_column: 1,
            end_line: line,
            end_column: 10,
            text: String::new(),
            children: vec![],
            parent,
            metadata: HashMap::new(),
        }
    }

    fn create_list_token(line: usize, children: Vec<usize>) -> Token {
        Token {
            token_type: "list".to_string(),
            start_line: line,
            start_column: 1,
            end_line: line + 2,
            end_column: 1,
            text: String::new(),
            children,
            parent: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_md029_consistent_one() {
        let lines = vec![
            "1. Item 1\n".to_string(),
            "1. Item 2\n".to_string(),
            "1. Item 3\n".to_string(),
        ];

        let tokens = vec![
            create_list_token(1, vec![1, 2, 3]),
            create_list_item_token(1, Some(0)),
            create_list_item_token(2, Some(0)),
            create_list_item_token(3, Some(0)),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD029;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md029_consistent_ordered() {
        let lines = vec![
            "1. Item 1\n".to_string(),
            "2. Item 2\n".to_string(),
            "3. Item 3\n".to_string(),
        ];

        let tokens = vec![
            create_list_token(1, vec![1, 2, 3]),
            create_list_item_token(1, Some(0)),
            create_list_item_token(2, Some(0)),
            create_list_item_token(3, Some(0)),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD029;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md029_inconsistent_mixed() {
        let lines = vec![
            "1. Item 1\n".to_string(),
            "1. Item 2\n".to_string(),
            "2. Item 3\n".to_string(), // Should be 1
        ];

        let tokens = vec![
            create_list_token(1, vec![1, 2, 3]),
            create_list_item_token(1, Some(0)),
            create_list_item_token(2, Some(0)),
            create_list_item_token(3, Some(0)),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD029;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Expected: 1")
        );
    }

    #[test]
    fn test_md029_style_ordered() {
        let lines = vec![
            "1. Item 1\n".to_string(),
            "1. Item 2\n".to_string(), // Should be 2
            "1. Item 3\n".to_string(), // Should be 3
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), serde_json::json!("ordered"));

        let tokens = vec![
            create_list_token(1, vec![1, 2, 3]),
            create_list_item_token(1, Some(0)),
            create_list_item_token(2, Some(0)),
            create_list_item_token(3, Some(0)),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD029;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(errors[1].line_number, 3);
    }

    #[test]
    fn test_md029_style_one() {
        let lines = vec![
            "1. Item 1\n".to_string(),
            "2. Item 2\n".to_string(), // Should be 1
            "3. Item 3\n".to_string(), // Should be 1
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), serde_json::json!("one"));

        let tokens = vec![
            create_list_token(1, vec![1, 2, 3]),
            create_list_item_token(1, Some(0)),
            create_list_item_token(2, Some(0)),
            create_list_item_token(3, Some(0)),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD029;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(errors[1].line_number, 3);
    }

    #[test]
    fn test_md029_style_zero() {
        let lines = vec![
            "0. Item 1\n".to_string(),
            "0. Item 2\n".to_string(),
            "0. Item 3\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), serde_json::json!("zero"));

        let tokens = vec![
            create_list_token(1, vec![1, 2, 3]),
            create_list_item_token(1, Some(0)),
            create_list_item_token(2, Some(0)),
            create_list_item_token(3, Some(0)),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD029;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md029_auto_detect_zero_increment() {
        let lines = vec![
            "0. Item 1\n".to_string(),
            "1. Item 2\n".to_string(),
            "2. Item 3\n".to_string(),
        ];

        let tokens = vec![
            create_list_token(1, vec![1, 2, 3]),
            create_list_item_token(1, Some(0)),
            create_list_item_token(2, Some(0)),
            create_list_item_token(3, Some(0)),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD029;
        let errors = rule.lint(&params);
        // Should auto-detect as ordered starting from 0
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_get_ordered_list_value() {
        assert_eq!(get_ordered_list_value("1. Item"), Some((1, 1, 1)));
        assert_eq!(get_ordered_list_value("2. Item"), Some((2, 1, 1)));
        assert_eq!(get_ordered_list_value("10. Item"), Some((10, 1, 2)));
        assert_eq!(get_ordered_list_value("  3. Item"), Some((3, 3, 1)));
        assert_eq!(get_ordered_list_value("0. Item"), Some((0, 1, 1)));
        assert_eq!(get_ordered_list_value("- Item"), None);
        assert_eq!(get_ordered_list_value("Not a list"), None);
    }
}
