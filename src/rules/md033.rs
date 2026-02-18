//! MD033 - Inline HTML
//!
//! This rule checks for inline HTML elements in the markdown content.
//! It can be configured to allow specific HTML elements.

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use std::sync::LazyLock;

static HTML_TAG_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^<([^!>][^/\s>]*)").expect("valid regex"));

pub struct MD033;

/// Extract HTML tag information from a token
struct HtmlTagInfo {
    name: String,
    close: bool,
}

fn get_html_tag_info(text: &str) -> Option<HtmlTagInfo> {
    if let Some(captures) = HTML_TAG_NAME_RE.captures(text)
        && let Some(name_match) = captures.get(1)
    {
        let mut name = name_match.as_str();
        let close = name.starts_with('/');

        // Strip leading '/' for closing tags
        if close {
            name = &name[1..];
        }

        // Strip trailing '/' for self-closing tags like <br/>
        let name = name.trim_end_matches('/');

        return Some(HtmlTagInfo {
            name: name.to_string(),
            close,
        });
    }
    None
}

/// Check if a token has a parent of the specified type
fn has_parent_of_type(
    tokens: &[crate::parser::Token],
    token_idx: usize,
    parent_type: &str,
) -> bool {
    if let Some(token) = tokens.get(token_idx)
        && let Some(parent_idx) = token.parent
        && let Some(parent) = tokens.get(parent_idx)
    {
        if parent.token_type == parent_type {
            return true;
        }
        // Recursively check parent's parent
        return has_parent_of_type(tokens, parent_idx, parent_type);
    }
    false
}

/// Convert config value to lowercase string array
fn to_lowercase_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    if let Some(val) = value
        && let Some(arr) = val.as_array()
    {
        return arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_lowercase())
            .collect();
    }
    Vec::new()
}

impl Rule for MD033 {
    fn names(&self) -> &'static [&'static str] {
        &["MD033", "no-inline-html"]
    }

    fn description(&self) -> &'static str {
        "Inline HTML"
    }

    fn tags(&self) -> &[&'static str] {
        &["html"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md033.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get configuration
        let allowed_elements = to_lowercase_string_array(params.config.get("allowed_elements"));

        // If not defined, use allowed_elements for backward compatibility
        let table_allowed_elements = if params.config.contains_key("table_allowed_elements") {
            to_lowercase_string_array(params.config.get("table_allowed_elements"))
        } else {
            allowed_elements.clone()
        };

        for (idx, token) in params.tokens.iter().enumerate() {
            if token.token_type != "htmlText" {
                continue;
            }

            // Get HTML tag info
            if let Some(html_tag_info) = get_html_tag_info(&token.text) {
                // Skip closing tags
                if html_tag_info.close {
                    continue;
                }

                let element_name = html_tag_info.name.to_lowercase();
                let in_table = has_parent_of_type(params.tokens, idx, "table");

                // Check if element should trigger an error
                // Logic from JS: (inTable || !allowedElements.includes(elementName)) && (!inTable || !tableAllowedElements.includes(elementName))
                let should_error = (in_table || !allowed_elements.contains(&element_name))
                    && (!in_table || !table_allowed_elements.contains(&element_name));

                if should_error {
                    // Calculate range - first line only
                    let first_line_text = token.text.lines().next().unwrap_or(&token.text);
                    let range = (token.start_column, first_line_text.len());

                    errors.push(LintError {
                        line_number: token.start_line,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!("Element: {}", html_tag_info.name)),
                        error_context: None,
                        rule_information: self.information(),
                        error_range: Some(range),
                        fix_info: None,
                        suggestion: Some("Avoid using raw HTML in Markdown".to_string()),
                        severity: Severity::Error,
                        fix_only: false,
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
    use std::collections::HashMap;

    #[test]
    fn test_get_html_tag_info() {
        let info = get_html_tag_info("<div>");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "div");
        assert!(!info.close);

        let info = get_html_tag_info("</div>");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "div");
        assert!(info.close);

        let info = get_html_tag_info("<br/>");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "br"); // Self-closing tags should have the tag name without '/'
        assert!(!info.close);

        let info = get_html_tag_info("<!-- comment -->");
        assert!(info.is_none());
    }

    #[test]
    fn test_md033_no_html() {
        let tokens = vec![];
        let lines = vec!["# Heading\n", "Some text\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD033;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md033_with_html() {
        let tokens = vec![Token {
            token_type: "htmlText".to_string(),
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 6,
            text: "<div>".to_string(),
            children: vec![],
            parent: None,
            metadata: HashMap::new(),
        }];

        let lines = vec!["<div>\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD033;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_detail, Some("Element: div".to_string()));
    }

    #[test]
    fn test_md033_with_allowed_elements() {
        let tokens = vec![Token {
            token_type: "htmlText".to_string(),
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 6,
            text: "<div>".to_string(),
            children: vec![],
            parent: None,
            metadata: HashMap::new(),
        }];

        let lines = vec!["<div>\n"];

        let mut config = HashMap::new();
        config.insert("allowed_elements".to_string(), serde_json::json!(["div"]));

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
            workspace_headings: None,
        };

        let rule = MD033;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md033_closing_tag_ignored() {
        let tokens = vec![Token {
            token_type: "htmlText".to_string(),
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 7,
            text: "</div>".to_string(),
            children: vec![],
            parent: None,
            metadata: HashMap::new(),
        }];

        let lines = vec!["</div>\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD033;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
