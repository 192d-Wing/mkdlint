//! MD036 - Emphasis used instead of a heading
//!
//! This rule detects when emphasis (bold or italic) is used for what should be a heading.
//! It looks for single-line paragraphs that consist entirely of emphasized text and don't
//! end with punctuation.
//!
//! ## Parameters
//!
//! - `punctuation`: Characters to treat as punctuation (default: `.,;:!?。，；：！？`)

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD036;

/// Default punctuation characters
const ALL_PUNCTUATION: &str = ".,;:!?。，；：！？";

/// Check if a paragraph child token is meaningful
/// (i.e., not just HTML or whitespace)
fn is_paragraph_child_meaningful(token: &crate::parser::Token) -> bool {
    !(token.token_type == "htmlText"
        || (token.token_type == "data" && token.text.trim().is_empty()))
}

impl Rule for MD036 {
    fn names(&self) -> &[&'static str] {
        &["MD036", "no-emphasis-as-heading"]
    }

    fn description(&self) -> &'static str {
        "Emphasis used instead of a heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "emphasis", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md036.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get punctuation from config or use default
        let punctuation = params
            .config
            .get("punctuation")
            .and_then(|v| v.as_str())
            .unwrap_or(ALL_PUNCTUATION);

        // Create regex pattern to match punctuation at end of string
        let punctuation_pattern = format!("[{}]$", regex::escape(punctuation));
        let punctuation_re = match regex::Regex::new(&punctuation_pattern) {
            Ok(re) => re,
            Err(_) => return errors, // Return empty if regex fails
        };

        // Find all paragraph tokens
        let paragraphs = params.tokens.filter_by_type("paragraph");

        // Filter paragraphs based on JS logic:
        // - parent is "content"
        // - parent has no parent OR parent.parent is "htmlFlow" with no parent
        // - has exactly one meaningful child
        let filtered_paragraphs: Vec<_> = paragraphs
            .iter()
            .filter(|para| {
                // Check parent is "content"
                if let Some(parent_idx) = para.parent
                    && let Some(parent) = params.tokens.get(parent_idx)
                {
                    if parent.token_type != "content" {
                        return false;
                    }

                    // Check parent has no parent OR parent.parent is htmlFlow with no parent
                    if let Some(grandparent_idx) = parent.parent {
                        if let Some(grandparent) = params.tokens.get(grandparent_idx) {
                            if grandparent.token_type != "htmlFlow" {
                                return false;
                            }
                            // htmlFlow should have no parent
                            if grandparent.parent.is_some() {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }

                    // Check for exactly one meaningful child
                    let meaningful_children: Vec<_> = para
                        .children
                        .iter()
                        .filter_map(|&child_idx| params.tokens.get(child_idx))
                        .filter(|child| is_paragraph_child_meaningful(child))
                        .collect();

                    return meaningful_children.len() == 1;
                }
                false
            })
            .collect();

        // Check both emphasis and strong types
        let emphasis_types = [["emphasis", "emphasisText"], ["strong", "strongText"]];

        for emphasis_type in &emphasis_types {
            // Get descendants of filtered paragraphs matching the emphasis types
            for paragraph in &filtered_paragraphs {
                // Recursively search for emphasis/strong tokens
                let text_tokens = get_descendants_by_types(params.tokens, paragraph, emphasis_type);

                for text_token in text_tokens {
                    // Check if:
                    // 1. Has exactly one child
                    // 2. That child is of type "data"
                    // 3. Text doesn't end in punctuation
                    if text_token.children.len() == 1
                        && let Some(&child_idx) = text_token.children.first()
                        && let Some(child) = params.tokens.get(child_idx)
                        && child.token_type == "data"
                        && !punctuation_re.is_match(&text_token.text)
                    {
                        // Find parent emphasis/strong token to get full range with markers
                        let parent_token = text_token
                            .parent
                            .and_then(|p_idx| params.tokens.get(p_idx))
                            .filter(|p| p.token_type == emphasis_type[0]);

                        let fix_info = if let Some(parent) = parent_token {
                            // Full range from start of parent (including opening marker)
                            // to end of parent (including closing marker)
                            let start_col = parent.start_column;
                            let end_col = parent.end_column;
                            let total_len = end_col - start_col;

                            Some(FixInfo {
                                line_number: None,
                                edit_column: Some(start_col),
                                delete_count: Some(total_len as i32),
                                insert_text: Some(format!("## {}", text_token.text)),
                            })
                        } else {
                            None
                        };

                        errors.push(LintError {
                            line_number: text_token.start_line,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: None,
                            error_context: Some(text_token.text.clone()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: None,
                            fix_info,
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        errors
    }
}

/// Get all descendant tokens matching any of the given types (recursive search)
fn get_descendants_by_types<'a>(
    all_tokens: &'a [crate::parser::Token],
    parent: &crate::parser::Token,
    types: &[&str],
) -> Vec<&'a crate::parser::Token> {
    let mut results = Vec::new();
    let mut to_visit = parent.children.clone();

    while let Some(idx) = to_visit.pop() {
        if let Some(token) = all_tokens.get(idx) {
            // Check if this token matches any of the types
            if types.iter().any(|t| token.token_type == *t) {
                results.push(token);
            }
            // Add children to visit list
            to_visit.extend(&token.children);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Token;
    use std::collections::HashMap;

    fn create_token(
        token_type: &str,
        start_line: usize,
        text: &str,
        children: Vec<usize>,
        parent: Option<usize>,
    ) -> Token {
        Token {
            token_type: token_type.to_string(),
            start_line,
            start_column: 1,
            end_line: start_line,
            end_column: text.len(),
            text: text.to_string(),
            children,
            parent,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_md036_emphasis_as_heading() {
        // Create a simple token structure:
        // content (0) -> paragraph (1) -> emphasis (2) -> emphasisText (3) -> data (4)
        let mut tokens = vec![
            create_token("content", 1, "", vec![1], None),
            create_token("paragraph", 1, "", vec![2], Some(0)),
            create_token("emphasis", 1, "", vec![3], Some(1)),
            create_token("emphasisText", 1, "Heading", vec![4], Some(2)),
            create_token("data", 1, "Heading", vec![], Some(3)),
        ];
        // Set proper column positions for emphasis token
        tokens[2].start_column = 1;
        tokens[2].end_column = 10;

        let lines = vec!["_Heading_\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD036;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
        assert_eq!(errors[0].error_context, Some("Heading".to_string()));
    }

    #[test]
    fn test_md036_strong_as_heading() {
        // Create a simple token structure:
        // content (0) -> paragraph (1) -> strong (2) -> strongText (3) -> data (4)
        let mut tokens = vec![
            create_token("content", 1, "", vec![1], None),
            create_token("paragraph", 1, "", vec![2], Some(0)),
            create_token("strong", 1, "", vec![3], Some(1)),
            create_token("strongText", 1, "Heading", vec![4], Some(2)),
            create_token("data", 1, "Heading", vec![], Some(3)),
        ];
        // Set proper column positions for strong token
        tokens[2].start_column = 1;
        tokens[2].end_column = 13;

        let lines = vec!["**Heading**\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD036;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
    }

    #[test]
    fn test_md036_with_punctuation() {
        // Emphasis with punctuation should NOT trigger
        let tokens = vec![
            create_token("content", 1, "", vec![1], None),
            create_token("paragraph", 1, "", vec![2], Some(0)),
            create_token("emphasis", 1, "", vec![3], Some(1)),
            create_token("emphasisText", 1, "Not a heading.", vec![4], Some(2)),
            create_token("data", 1, "Not a heading.", vec![], Some(3)),
        ];

        let lines = vec!["_Not a heading._\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD036;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md036_normal_text() {
        // Regular paragraph should not trigger
        let tokens = vec![
            create_token("content", 1, "", vec![1], None),
            create_token("paragraph", 1, "", vec![2], Some(0)),
            create_token("data", 1, "Normal text", vec![], Some(1)),
        ];

        let lines = vec!["Normal text\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD036;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md036_fix_emphasis_to_heading() {
        // Create token structure with proper parent relationships
        // emphasis (2) is parent of emphasisText (3)
        let mut tokens = vec![
            create_token("content", 1, "", vec![1], None),
            create_token("paragraph", 1, "", vec![2], Some(0)),
            create_token("emphasis", 1, "", vec![3], Some(1)),
            create_token("emphasisText", 1, "Heading", vec![4], Some(2)),
            create_token("data", 1, "Heading", vec![], Some(3)),
        ];
        // Set proper column positions for emphasis token (includes markers)
        tokens[2].start_column = 1;
        tokens[2].end_column = 10; // "_Heading_" = 9 chars + 1 for end position

        let lines = vec!["_Heading_\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD036;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(9)); // Full length
        assert_eq!(fix.insert_text, Some("## Heading".to_string()));
    }

    #[test]
    fn test_md036_fix_strong_to_heading() {
        // Create token structure with proper parent relationships
        let mut tokens = vec![
            create_token("content", 1, "", vec![1], None),
            create_token("paragraph", 1, "", vec![2], Some(0)),
            create_token("strong", 1, "", vec![3], Some(1)),
            create_token("strongText", 1, "Heading", vec![4], Some(2)),
            create_token("data", 1, "Heading", vec![], Some(3)),
        ];
        // Set proper column positions for strong token (includes markers)
        tokens[2].start_column = 1;
        tokens[2].end_column = 13; // "**Heading**" = 11 chars + 1 for end position

        let lines = vec!["**Heading**\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD036;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(12)); // Full length
        assert_eq!(fix.insert_text, Some("## Heading".to_string()));
    }
}
