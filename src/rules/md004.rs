//! MD004 - Unordered list style
//!
//! This rule checks that unordered list markers are consistent throughout the document.
//! Supported styles:
//! - `asterisk`: All markers should be `*`
//! - `dash`: All markers should be `-`
//! - `plus`: All markers should be `+`
//! - `consistent`: All markers should be the same (default)
//! - `sublist`: Sublists should use a different marker than their parent

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use std::collections::HashMap;

pub struct MD004;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListStyle {
    Asterisk,
    Dash,
    Plus,
    Consistent,
    Sublist,
}

impl ListStyle {
    fn from_str(s: &str) -> Self {
        match s {
            "asterisk" => ListStyle::Asterisk,
            "dash" => ListStyle::Dash,
            "plus" => ListStyle::Plus,
            "sublist" => ListStyle::Sublist,
            _ => ListStyle::Consistent,
        }
    }

    fn to_marker(self) -> char {
        match self {
            ListStyle::Asterisk => '*',
            ListStyle::Dash => '-',
            ListStyle::Plus => '+',
            ListStyle::Consistent | ListStyle::Sublist => '-', // Default fallback
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            ListStyle::Asterisk => "asterisk",
            ListStyle::Dash => "dash",
            ListStyle::Plus => "plus",
            ListStyle::Consistent => "consistent",
            ListStyle::Sublist => "sublist",
        }
    }
}

fn marker_to_style(marker: char) -> ListStyle {
    match marker {
        '*' => ListStyle::Asterisk,
        '-' => ListStyle::Dash,
        '+' => ListStyle::Plus,
        _ => ListStyle::Dash, // Default fallback
    }
}

fn different_item_style(style: ListStyle) -> ListStyle {
    match style {
        ListStyle::Dash => ListStyle::Plus,
        ListStyle::Plus => ListStyle::Asterisk,
        ListStyle::Asterisk => ListStyle::Dash,
        _ => ListStyle::Dash,
    }
}

/// Extract the list marker character from a line
fn get_list_marker(line: &str) -> Option<char> {
    let trimmed = line.trim_start();
    if let Some(first_char) = trimmed.chars().next()
        && matches!(first_char, '*' | '-' | '+')
    {
        // Make sure it's followed by whitespace (to distinguish from other uses of these chars)
        if trimmed.len() > 1 {
            let second_char = trimmed.chars().nth(1)?;
            if second_char.is_whitespace() {
                return Some(first_char);
            }
        }
    }
    None
}

/// Count nesting level by examining parent list tokens
fn get_nesting_level(
    tokens: &[crate::parser::Token],
    current_token: &crate::parser::Token,
) -> usize {
    let mut level = 0;
    let mut current = current_token;

    while let Some(parent_idx) = current.parent {
        if let Some(parent) = tokens.get(parent_idx) {
            // Check if parent is a list (ordered or unordered)
            if parent.token_type == "list" {
                level += 1;
            }
            current = parent;
        } else {
            break;
        }
    }

    level
}

impl Rule for MD004 {
    fn names(&self) -> &[&'static str] {
        &["MD004", "ul-style"]
    }

    fn description(&self) -> &'static str {
        "Unordered list style"
    }

    fn tags(&self) -> &[&'static str] {
        &["bullet", "ul"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md004.md")
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

        let mut expected_style = style;
        let mut nesting_styles: HashMap<usize, ListStyle> = HashMap::new();

        // Find all list items
        let list_items = params.tokens.filter_by_type("listItem");

        for item in list_items {
            // Only process unordered lists - check by looking at the actual marker in the line
            if item.start_line == 0 || item.start_line > params.lines.len() {
                continue;
            }

            let line = &params.lines[item.start_line - 1];

            // Extract the marker from the line
            if let Some(marker) = get_list_marker(line) {
                let item_style = marker_to_style(marker);

                // Handle sublist style
                let nesting = if style == ListStyle::Sublist {
                    get_nesting_level(params.tokens, item)
                } else {
                    0
                };

                if style == ListStyle::Sublist {
                    // Get or set expected style for this nesting level
                    if let Some(&nested_expected) = nesting_styles.get(&nesting) {
                        expected_style = nested_expected;
                    } else {
                        // Set expected style for this level
                        expected_style = if nesting > 0 {
                            // Check parent level
                            if let Some(&parent_style) = nesting_styles.get(&(nesting - 1)) {
                                if item_style == parent_style {
                                    different_item_style(item_style)
                                } else {
                                    item_style
                                }
                            } else {
                                item_style
                            }
                        } else {
                            item_style
                        };
                        nesting_styles.insert(nesting, expected_style);
                    }
                } else if expected_style == ListStyle::Consistent {
                    // Set the expected style to the first item's style
                    expected_style = item_style;
                }

                // Check if the item matches expected style
                if item_style != expected_style {
                    // Find the column where the marker appears
                    let marker_pos = line
                        .chars()
                        .position(|c| matches!(c, '*' | '-' | '+'))
                        .unwrap_or(0);
                    let column = marker_pos + 1;

                    errors.push(LintError {
                        line_number: item.start_line,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!(
                            "Expected: {}; Actual: {}",
                            expected_style.to_str(),
                            item_style.to_str()
                        )),
                        error_context: Some(format!("{}", marker)),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: Some((column, 1)),
                        fix_info: Some(FixInfo {
                            line_number: None,
                            edit_column: Some(column),
                            delete_count: Some(1),
                            insert_text: Some(expected_style.to_marker().to_string()),
                        }),
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
    use crate::parser::Token;

    fn create_list_item_token(line: usize) -> Token {
        Token {
            token_type: "listItem".to_string(),
            start_line: line,
            start_column: 1,
            end_line: line,
            end_column: 10,
            text: String::new(),
            children: vec![],
            parent: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_md004_consistent_asterisk() {
        let tokens = vec![
            create_list_item_token(1),
            create_list_item_token(2),
            create_list_item_token(3),
        ];

        let lines = vec![
            "* Item 1\n".to_string(),
            "* Item 2\n".to_string(),
            "* Item 3\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD004;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md004_consistent_dash() {
        let tokens = vec![
            create_list_item_token(1),
            create_list_item_token(2),
            create_list_item_token(3),
        ];

        let lines = vec![
            "- Item 1\n".to_string(),
            "- Item 2\n".to_string(),
            "- Item 3\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD004;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md004_inconsistent_markers() {
        let tokens = vec![
            create_list_item_token(1),
            create_list_item_token(2),
            create_list_item_token(3),
        ];

        let lines = vec![
            "* Item 1\n".to_string(),
            "- Item 2\n".to_string(),
            "+ Item 3\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD004;
        let errors = rule.lint(&params);
        // First item sets the style, so items 2 and 3 should error
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(errors[1].line_number, 3);
    }

    #[test]
    fn test_md004_style_dash() {
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("dash".to_string()),
        );

        let tokens = vec![create_list_item_token(1), create_list_item_token(2)];

        let lines = vec!["* Item 1\n".to_string(), "- Item 2\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD004;
        let errors = rule.lint(&params);
        // Item 1 should error (asterisk instead of dash)
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert!(errors[0].error_detail.as_ref().unwrap().contains("dash"));
    }

    #[test]
    fn test_md004_style_asterisk() {
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("asterisk".to_string()),
        );

        let tokens = vec![create_list_item_token(1), create_list_item_token(2)];

        let lines = vec!["- Item 1\n".to_string(), "- Item 2\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD004;
        let errors = rule.lint(&params);
        // Both items should error
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md004_style_plus() {
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("plus".to_string()),
        );

        let tokens = vec![create_list_item_token(1), create_list_item_token(2)];

        let lines = vec!["+ Item 1\n".to_string(), "+ Item 2\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD004;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_get_list_marker() {
        assert_eq!(get_list_marker("* Item"), Some('*'));
        assert_eq!(get_list_marker("- Item"), Some('-'));
        assert_eq!(get_list_marker("+ Item"), Some('+'));
        assert_eq!(get_list_marker("  * Indented"), Some('*'));
        assert_eq!(get_list_marker("Not a list"), None);
        assert_eq!(get_list_marker("1. Ordered list"), None);
    }
}
