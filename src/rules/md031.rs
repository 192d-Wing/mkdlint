//! MD031 - Fenced code blocks should be surrounded by blank lines

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static CODE_FENCE_PREFIX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.*?)[`~]").unwrap());

pub struct MD031;

/// Check if a line is blank (empty or whitespace only)
fn is_blank_line(line: &str) -> bool {
    line.trim().is_empty()
}

/// Extract the prefix (indentation and blockquote markers) from a code fence line
fn get_code_fence_prefix(line: &str) -> Option<String> {
    CODE_FENCE_PREFIX_RE.captures(line).map(|caps| {
        caps.get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    })
}

/// Check if a line is inside a list item based on indentation
fn is_in_list_context(lines: &[&str], start_idx: usize) -> bool {
    // Look backward to find if we're in a list context
    // A simple heuristic: check if there's a list marker in previous lines
    // with less or equal indentation
    if start_idx == 0 {
        return false;
    }

    let fence_line = &lines[start_idx];
    let fence_indent = fence_line.len() - fence_line.trim_start().len();

    // Look back up to 10 lines
    let start_search = start_idx.saturating_sub(10);

    for i in (start_search..start_idx).rev() {
        let line = &lines[i];
        let trimmed = line.trim_start();

        // Check for list markers: -, *, +, or numbered lists
        if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
            || (trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                && trimmed.contains(". "))
        {
            let line_indent = line.len() - trimmed.len();
            if line_indent <= fence_indent {
                return true;
            }
        }

        // Stop at blank lines that might separate list contexts
        if is_blank_line(line) && i < start_idx - 1 {
            break;
        }
    }

    false
}

impl Rule for MD031 {
    fn names(&self) -> &'static [&'static str] {
        &["MD031", "blanks-around-fences"]
    }

    fn description(&self) -> &'static str {
        "Fenced code blocks should be surrounded by blank lines"
    }

    fn tags(&self) -> &[&'static str] {
        &["code", "blank_lines", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md031.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Check config for list_items option (default: true)
        let list_items = params
            .config
            .get("list_items")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let lines = params.lines;
        let mut in_code_fence = false;
        let mut fence_start_line = 0;
        let mut fence_char = '\0';

        for (idx, line) in lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_start();

            // Check if this line starts or ends a code fence
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let current_fence_char = trimmed.chars().next().unwrap();

                if !in_code_fence {
                    // Starting a new code fence
                    in_code_fence = true;
                    fence_start_line = line_number;
                    fence_char = current_fence_char;

                    // Check if we should skip list items
                    if !list_items && is_in_list_context(lines, idx) {
                        continue;
                    }

                    // Check for blank line before fence
                    if idx > 0 && !is_blank_line(lines[idx - 1]) {
                        // Get the prefix for fix info
                        let prefix = get_code_fence_prefix(line).unwrap_or_default();
                        let insert_text = if prefix.is_empty() {
                            "\n".to_string()
                        } else {
                            // Replace non-blockquote chars with spaces and trim
                            let mut fixed_prefix = String::new();
                            for ch in prefix.chars() {
                                if ch == '>' {
                                    fixed_prefix.push(ch);
                                } else {
                                    fixed_prefix.push(' ');
                                }
                            }
                            format!("{}\n", fixed_prefix.trim())
                        };

                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: None,
                            error_context: Some(line.trim().to_string()),
                            rule_information: self.information(),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: Some(line_number),
                                edit_column: Some(1),
                                delete_count: None,
                                insert_text: Some(insert_text),
                            }),
                            suggestion: Some(
                                "Fenced code blocks should be surrounded by blank lines"
                                    .to_string(),
                            ),
                            severity: Severity::Error,
                        });
                    }
                } else if current_fence_char == fence_char {
                    // Check if this could be a closing fence
                    // Count the fence characters
                    let fence_count = trimmed.chars().take_while(|&c| c == fence_char).count();

                    // Only treat as closing if it has at least 3 fence chars and nothing else
                    // (or just fence chars followed by whitespace)
                    let rest = &trimmed[fence_count..];
                    if fence_count >= 3 && rest.trim().is_empty() {
                        // Closing the code fence
                        in_code_fence = false;

                        // Check if we should skip list items
                        if !list_items && is_in_list_context(lines, fence_start_line - 1) {
                            continue;
                        }

                        // Check for blank line after fence
                        if idx + 1 < lines.len() && !is_blank_line(lines[idx + 1]) {
                            // Get the prefix for fix info
                            let prefix = get_code_fence_prefix(line).unwrap_or_default();
                            let insert_text = if prefix.is_empty() {
                                "\n".to_string()
                            } else {
                                // Replace non-blockquote chars with spaces and trim
                                let mut fixed_prefix = String::new();
                                for ch in prefix.chars() {
                                    if ch == '>' {
                                        fixed_prefix.push(ch);
                                    } else {
                                        fixed_prefix.push(' ');
                                    }
                                }
                                format!("{}\n", fixed_prefix.trim())
                            };

                            errors.push(LintError {
                                line_number,
                                rule_names: self.names(),
                                rule_description: self.description(),
                                error_detail: None,
                                error_context: Some(line.trim().to_string()),
                                rule_information: self.information(),
                                error_range: None,
                                fix_info: Some(FixInfo {
                                    line_number: Some(line_number + 1),
                                    edit_column: Some(1),
                                    delete_count: None,
                                    insert_text: Some(insert_text),
                                }),
                                suggestion: Some(
                                    "Fenced code blocks should be surrounded by blank lines"
                                        .to_string(),
                                ),
                                severity: Severity::Error,
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
    use std::collections::HashMap;

    #[test]
    fn test_md031_valid_blank_lines() {
        let lines = vec![
            "# Heading\n",
            "\n",
            "```rust\n",
            "let x = 5;\n",
            "```\n",
            "\n",
            "More text\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD031;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md031_missing_blank_before() {
        let lines = vec!["# Heading\n", "```rust\n", "let x = 5;\n", "```\n", "\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD031;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2); // Opening fence line
    }

    #[test]
    fn test_md031_missing_blank_after() {
        let lines = vec![
            "# Heading\n",
            "\n",
            "```rust\n",
            "let x = 5;\n",
            "```\n",
            "More text\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD031;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 5); // Closing fence line
    }

    #[test]
    fn test_md031_missing_both_blanks() {
        let lines = vec![
            "# Heading\n",
            "```rust\n",
            "let x = 5;\n",
            "```\n",
            "More text\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD031;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 2); // Opening fence
        assert_eq!(errors[1].line_number, 4); // Closing fence
    }

    #[test]
    fn test_md031_tilde_fences() {
        let lines = vec!["Text\n", "~~~\n", "code\n", "~~~\n", "Text\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD031;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2); // Missing blank before and after
    }

    #[test]
    fn test_md031_start_of_file() {
        let lines = vec!["```rust\n", "let x = 5;\n", "```\n", "\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD031;
        let errors = rule.lint(&params);
        // No error for missing blank before when at start of file
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md031_end_of_file() {
        let lines = vec!["\n", "```rust\n", "let x = 5;\n", "```\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD031;
        let errors = rule.lint(&params);
        // No error for missing blank after when at end of file
        assert_eq!(errors.len(), 0);
    }
}
