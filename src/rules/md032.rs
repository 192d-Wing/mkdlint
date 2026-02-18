//! MD032 - Lists should be surrounded by blank lines

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use std::collections::HashSet;

pub struct MD032;

/// Check if a line is blank (empty or contains only whitespace/comments)
fn is_blank_line(line: &str) -> bool {
    let mut s = line.to_string();

    // Remove HTML comments (simplified version of the JS implementation)
    loop {
        let start_comment = "<!--";
        let end_comment = "-->";

        let start = s.find(start_comment);
        let end = s.find(end_comment);

        match (start, end) {
            (None, Some(end_pos)) => {
                // Unmatched end comment is first
                s = s[end_pos + end_comment.len()..].to_string();
            }
            (Some(start_pos), Some(end_pos)) if start_pos < end_pos => {
                // Start comment is before end comment
                s = format!("{}{}", &s[..start_pos], &s[end_pos + end_comment.len()..]);
            }
            (Some(start_pos), None) => {
                // Unmatched start comment is last
                s = s[..start_pos].to_string();
            }
            _ => break,
        }
    }

    // After removing comments, check if line is empty or contains only whitespace/angle brackets
    s.is_empty() || s.trim().is_empty() || s.replace('>', "").trim().is_empty()
}

/// Get blockquote prefix text for inserting blank lines
fn get_blockquote_prefix(tokens: &[crate::parser::Token], line_number: usize) -> String {
    // Filter tokens for blockQuotePrefix and linePrefix at the specified line
    let prefixes: Vec<&crate::parser::Token> = tokens
        .iter()
        .filter(|t| {
            (t.token_type == "blockQuotePrefix" || t.token_type == "linePrefix")
                && t.start_line == line_number
        })
        .collect();

    let mut result = String::new();
    for prefix in prefixes {
        result.push_str(&prefix.text);
    }

    // Trim trailing whitespace and add newline
    result.trim_end().to_string() + "\n"
}

/// Set of token types that do not contain actual content
fn non_content_tokens() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert("blockQuoteMarker");
    set.insert("blockQuotePrefix");
    set.insert("blockQuotePrefixWhitespace");
    set.insert("gfmFootnoteDefinitionIndent");
    set.insert("lineEnding");
    set.insert("lineEndingBlank");
    set.insert("linePrefix");
    set.insert("listItemIndent");
    set.insert("undefinedReference");
    set.insert("undefinedReferenceCollapsed");
    set.insert("undefinedReferenceFull");
    set.insert("undefinedReferenceShortcut");
    set
}

/// Filter tokens by predicate recursively
fn filter_by_predicate<F, G>(
    tokens: &[crate::parser::Token],
    token_indices: &[usize],
    allowed: &F,
    transform_children: &Option<G>,
) -> Vec<usize>
where
    F: Fn(&crate::parser::Token) -> bool,
    G: Fn(&crate::parser::Token) -> Vec<usize>,
{
    let mut result = Vec::new();
    let mut queue: Vec<Vec<usize>> = vec![token_indices.to_vec()];

    while let Some(current_indices) = queue.pop() {
        for &idx in &current_indices {
            if let Some(token) = tokens.get(idx) {
                if allowed(token) {
                    result.push(idx);
                }

                let children = if let Some(transform) = transform_children {
                    transform(token)
                } else {
                    token.children.clone()
                };

                if !children.is_empty() {
                    queue.push(children);
                }
            }
        }
    }

    result
}

/// Check if token is a list
fn is_list(token: &crate::parser::Token) -> bool {
    token.token_type == "listOrdered" || token.token_type == "listUnordered"
}

impl Rule for MD032 {
    fn names(&self) -> &'static [&'static str] {
        &["MD032", "blanks-around-lists"]
    }

    fn description(&self) -> &'static str {
        "Lists should be surrounded by blank lines"
    }

    fn tags(&self) -> &[&'static str] {
        &["bullet", "ul", "ol", "blank_lines", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md032.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let lines = params.lines;
        let tokens = params.tokens;

        // Find all top-level lists (not nested within other lists or htmlFlow)
        let all_indices: Vec<usize> = (0..tokens.len()).collect();

        let top_level_lists = filter_by_predicate(
            tokens,
            &all_indices,
            &is_list,
            &Some(|token: &crate::parser::Token| {
                // Don't descend into lists or htmlFlow
                if is_list(token) || token.token_type == "htmlFlow" {
                    vec![]
                } else {
                    token.children.clone()
                }
            }),
        );

        for &list_idx in &top_level_lists {
            if let Some(list) = tokens.get(list_idx) {
                // Check for blank line above the list
                let first_line_number = list.start_line;

                // Line numbers are 1-based, array indices are 0-based
                // Check if previous line (index first_line_number - 2) is blank
                if first_line_number > 1 {
                    let prev_line_idx = first_line_number - 2;
                    if prev_line_idx < lines.len() && !is_blank_line(lines[prev_line_idx]) {
                        let context = if first_line_number - 1 <= lines.len() {
                            lines[first_line_number - 1].trim().to_string()
                        } else {
                            String::new()
                        };

                        let insert_text = get_blockquote_prefix(tokens, first_line_number);

                        errors.push(LintError {
                            line_number: first_line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: None,
                            error_context: Some(context),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: Some(first_line_number),
                                edit_column: Some(1),
                                delete_count: None,
                                insert_text: Some(insert_text),
                            }),
                            suggestion: Some(
                                "Lists should be surrounded by blank lines".to_string(),
                            ),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }
                }

                // Find the "visual" end of the list by filtering out non-content tokens
                let non_content = non_content_tokens();
                let flattened_children = filter_by_predicate(
                    tokens,
                    &list.children,
                    &|token| !non_content.contains(token.token_type.as_str()),
                    &Some(|token: &crate::parser::Token| {
                        if non_content.contains(token.token_type.as_str()) {
                            vec![]
                        } else {
                            token.children.clone()
                        }
                    }),
                );

                let end_line = if !flattened_children.is_empty() {
                    if let Some(&last_idx) = flattened_children.last() {
                        if let Some(last_token) = tokens.get(last_idx) {
                            last_token.end_line
                        } else {
                            list.end_line
                        }
                    } else {
                        list.end_line
                    }
                } else {
                    list.end_line
                };

                // Check for blank line below the list
                let last_line_number = end_line;

                // Check if next line (index last_line_number) exists and is not blank
                if last_line_number < lines.len() && !is_blank_line(lines[last_line_number]) {
                    let context = if last_line_number > 0 && last_line_number - 1 < lines.len() {
                        lines[last_line_number - 1].trim().to_string()
                    } else {
                        String::new()
                    };

                    let insert_text = get_blockquote_prefix(tokens, last_line_number);

                    errors.push(LintError {
                        line_number: last_line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: None,
                        error_context: Some(context),
                        rule_information: self.information(),
                        error_range: None,
                        fix_info: Some(FixInfo {
                            line_number: Some(last_line_number + 1),
                            edit_column: Some(1),
                            delete_count: None,
                            insert_text: Some(insert_text),
                        }),
                        suggestion: Some("Lists should be surrounded by blank lines".to_string()),
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
            end_column: 10,
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
        parent: usize,
    ) -> Token {
        Token {
            token_type: "listItem".to_string(),
            start_line,
            start_column: 1,
            end_line,
            end_column: 10,
            text: String::new(),
            children,
            parent: Some(parent),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_is_blank_line() {
        assert!(is_blank_line(""));
        assert!(is_blank_line("   "));
        assert!(is_blank_line("\t\t"));
        assert!(is_blank_line("<!-- comment -->"));
        assert!(!is_blank_line("text"));
        assert!(!is_blank_line("  text  "));
    }

    #[test]
    fn test_md032_valid_blank_lines() {
        let lines = vec![
            "# Heading\n",
            "\n",
            "- Item 1\n",
            "- Item 2\n",
            "\n",
            "Paragraph\n",
        ];

        let tokens = vec![
            create_list_token("listUnordered", 3, 4, vec![1, 2]),
            create_list_item_token(3, 3, vec![], 0),
            create_list_item_token(4, 4, vec![], 0),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD032;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md032_missing_blank_before() {
        let lines = vec!["# Heading\n", "- Item 1\n", "- Item 2\n", "\n"];

        let tokens = vec![
            create_list_token("listUnordered", 2, 3, vec![1, 2]),
            create_list_item_token(2, 2, vec![], 0),
            create_list_item_token(3, 3, vec![], 0),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD032;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
    }

    #[test]
    fn test_md032_missing_blank_after() {
        let lines = vec!["\n", "- Item 1\n", "- Item 2\n", "Paragraph\n"];

        let tokens = vec![
            create_list_token("listUnordered", 2, 3, vec![1, 2]),
            create_list_item_token(2, 2, vec![], 0),
            create_list_item_token(3, 3, vec![], 0),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD032;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
    }

    #[test]
    fn test_md032_ordered_list() {
        let lines = vec!["Paragraph\n", "1. Item 1\n", "2. Item 2\n", "More text\n"];

        let tokens = vec![
            create_list_token("listOrdered", 2, 3, vec![1, 2]),
            create_list_item_token(2, 2, vec![], 0),
            create_list_item_token(3, 3, vec![], 0),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD032;
        let errors = rule.lint(&params);
        // Should find errors for missing blanks before and after
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md032_at_start_of_file() {
        let lines = vec!["- Item 1\n", "- Item 2\n", "\n"];

        let tokens = vec![
            create_list_token("listUnordered", 1, 2, vec![1, 2]),
            create_list_item_token(1, 1, vec![], 0),
            create_list_item_token(2, 2, vec![], 0),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD032;
        let errors = rule.lint(&params);
        // No error for missing blank before when at start of file
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md032_at_end_of_file() {
        let lines = vec!["\n", "- Item 1\n", "- Item 2\n"];

        let tokens = vec![
            create_list_token("listUnordered", 2, 3, vec![1, 2]),
            create_list_item_token(2, 2, vec![], 0),
            create_list_item_token(3, 3, vec![], 0),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD032;
        let errors = rule.lint(&params);
        // No error for missing blank after when at end of file
        assert_eq!(errors.len(), 0);
    }
}
