//! MD046 - Code block style
//!
//! Supports `style` config: "consistent" (default), "fenced", or "indented".
//! - "consistent": all code blocks must use the same style as the first one found
//! - "fenced": all code blocks must be fenced (``` or ~~~)
//! - "indented": all code blocks must be indented (4 spaces)

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static CODE_FENCE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\s*)(`{3,}|~{3,})").expect("valid regex"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockStyle {
    Fenced,
    Indented,
}

/// A detected code block with its style, line range, and content.
#[allow(dead_code)]
struct CodeBlock {
    style: BlockStyle,
    start_line: usize,
    end_line: usize,
    /// 1-based line numbers of content lines (between fences or indented lines)
    content_lines: Vec<usize>,
    /// Info string from fenced block (e.g., "rust" from ```rust)
    fence_info: Option<String>,
}

pub struct MD046;

impl Rule for MD046 {
    fn names(&self) -> &'static [&'static str] {
        &["MD046", "code-block-style"]
    }

    fn description(&self) -> &'static str {
        "Code block style"
    }

    fn tags(&self) -> &[&'static str] {
        &["code", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md046.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let style_str = params
            .config
            .get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("consistent");

        let required_style = match style_str {
            "fenced" => Some(BlockStyle::Fenced),
            "indented" => Some(BlockStyle::Indented),
            _ => None, // "consistent" — determined by first block
        };

        // Collect all code blocks
        let blocks = find_code_blocks(params.lines);

        if blocks.is_empty() {
            return Vec::new();
        }

        // Determine the expected style
        let expected = required_style.unwrap_or(blocks[0].style);

        let expected_label = match expected {
            BlockStyle::Fenced => "fenced",
            BlockStyle::Indented => "indented",
        };

        let mut errors = Vec::new();
        for block in &blocks {
            if block.style != expected {
                let actual_label = match block.style {
                    BlockStyle::Fenced => "fenced",
                    BlockStyle::Indented => "indented",
                };

                // Generate fix_info for the conversion
                let fix_info = generate_block_fix(params.lines, block, expected);

                errors.push(LintError {
                    line_number: block.start_line,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!(
                        "Expected: {}; Actual: {}",
                        expected_label, actual_label
                    )),
                    error_context: None,
                    rule_information: self.information(),
                    error_range: None,
                    fix_info,
                    suggestion: Some(format!("Use {} code block style", expected_label)),
                    severity: Severity::Error,
                    fix_only: false,
                });

                // Emit helper delete-line errors for remaining lines of the block.
                // These are fix-only helpers (not shown to users).
                for line_num in (block.start_line + 1)..=block.end_line {
                    errors.push(LintError {
                        line_number: line_num,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: None,
                        error_context: None,
                        rule_information: self.information(),
                        error_range: None,
                        fix_info: Some(FixInfo {
                            line_number: Some(line_num),
                            edit_column: Some(1),
                            delete_count: Some(-1),
                            insert_text: None,
                        }),
                        suggestion: None,
                        severity: Severity::Error,
                        fix_only: true,
                    });
                }
            }
        }

        errors
    }
}

/// Generate the primary fix_info for converting a code block to the target style.
/// The fix replaces the first line of the block with the entire new block content
/// (using embedded newlines). Helper delete-line errors handle remaining old lines.
fn generate_block_fix(lines: &[&str], block: &CodeBlock, target: BlockStyle) -> Option<FixInfo> {
    match (block.style, target) {
        (BlockStyle::Indented, BlockStyle::Fenced) => {
            // Indented -> Fenced: remove 4-space indent, wrap with ```
            let mut replacement = String::from("```\n");
            for &content_ln in &block.content_lines {
                let line = lines[content_ln - 1];
                let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                // Remove 4-space indent; blank lines within block may have < 4 spaces
                let unindented = if let Some(stripped) = trimmed.strip_prefix("    ") {
                    stripped
                } else {
                    trimmed
                };
                replacement.push_str(unindented);
                replacement.push('\n');
            }
            replacement.push_str("```");

            Some(FixInfo {
                line_number: Some(block.start_line),
                edit_column: Some(1),
                delete_count: Some(i32::MAX),
                insert_text: Some(replacement),
            })
        }
        (BlockStyle::Fenced, BlockStyle::Indented) => {
            // Fenced -> Indented: remove fences, add 4-space indent
            let mut replacement = String::new();
            if block.content_lines.is_empty() {
                // Empty fenced block -> single indented blank line
                replacement.push_str("    ");
            } else {
                for (i, &content_ln) in block.content_lines.iter().enumerate() {
                    let line = lines[content_ln - 1];
                    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                    replacement.push_str("    ");
                    replacement.push_str(trimmed);
                    if i < block.content_lines.len() - 1 {
                        replacement.push('\n');
                    }
                }
            }

            Some(FixInfo {
                line_number: Some(block.start_line),
                edit_column: Some(1),
                delete_count: Some(i32::MAX),
                insert_text: Some(replacement),
            })
        }
        _ => None, // Same style, no fix needed
    }
}

/// Find all code blocks in the document, returning their style, line range, and content.
fn find_code_blocks(lines: &[&str]) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut in_fenced = false;
    let mut fence_indent = 0;
    let mut fence_char = ' ';
    let mut fence_len = 0;
    let mut fenced_start = 0;
    let mut fenced_content: Vec<usize> = Vec::new();
    let mut fenced_info = String::new();

    let mut in_indented = false;
    let mut indented_start = 0;
    let mut indented_content: Vec<usize> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

        // Check for fenced code block delimiter
        if let Some(caps) = CODE_FENCE_RE.captures(trimmed) {
            let indent = caps.get(1).unwrap().as_str().len();
            let fence = caps.get(2).unwrap().as_str();
            let fc = fence.chars().next().unwrap();
            let fl = fence.len();

            if in_fenced {
                // Closing fence: must match char, >= length, and <= indent
                if fc == fence_char && fl >= fence_len && indent <= fence_indent {
                    blocks.push(CodeBlock {
                        style: BlockStyle::Fenced,
                        start_line: fenced_start,
                        end_line: line_number,
                        content_lines: fenced_content.clone(),
                        fence_info: if fenced_info.is_empty() {
                            None
                        } else {
                            Some(fenced_info.clone())
                        },
                    });
                    in_fenced = false;
                    fenced_content.clear();
                    fenced_info.clear();
                }
            } else {
                // Opening fence (only if indent < 4, per CommonMark)
                if indent < 4 {
                    // End any indented block first
                    if in_indented {
                        let end_line = indented_content.last().copied().unwrap_or(indented_start);
                        blocks.push(CodeBlock {
                            style: BlockStyle::Indented,
                            start_line: indented_start,
                            end_line,
                            content_lines: indented_content.clone(),
                            fence_info: None,
                        });
                        in_indented = false;
                        indented_content.clear();
                    }
                    in_fenced = true;
                    fence_indent = indent;
                    fence_char = fc;
                    fence_len = fl;
                    fenced_start = line_number;
                    fenced_content.clear();
                    // Extract info string (text after the fence marker)
                    let after_fence = trimmed[indent + fl..].trim().to_string();
                    fenced_info = after_fence;
                }
            }
            continue;
        }

        if in_fenced {
            fenced_content.push(line_number);
            continue;
        }

        // Check for indented code block (4+ spaces, not inside a list)
        // An indented code block requires a blank line before it (or start of doc)
        let is_indented_line = !trimmed.is_empty() && line.starts_with("    ");

        if is_indented_line {
            if !in_indented {
                // Check for blank line before (or start of document)
                let prev_blank = if idx == 0 {
                    true
                } else {
                    let prev = lines[idx - 1].trim_end_matches('\n').trim_end_matches('\r');
                    prev.trim().is_empty()
                };
                if prev_blank {
                    in_indented = true;
                    indented_start = line_number;
                }
            }
            if in_indented {
                indented_content.push(line_number);
            }
        } else {
            // Non-indented, non-empty line ends an indented block
            if in_indented && !trimmed.is_empty() {
                let end_line = indented_content.last().copied().unwrap_or(indented_start);
                blocks.push(CodeBlock {
                    style: BlockStyle::Indented,
                    start_line: indented_start,
                    end_line,
                    content_lines: indented_content.clone(),
                    fence_info: None,
                });
                in_indented = false;
                indented_content.clear();
            }
            // Blank lines don't end the indented block (they can appear within)
        }
    }

    // Close trailing indented block
    if in_indented {
        let end_line = indented_content.last().copied().unwrap_or(indented_start);
        blocks.push(CodeBlock {
            style: BlockStyle::Indented,
            start_line: indented_start,
            end_line,
            content_lines: indented_content.clone(),
            fence_info: None,
        });
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_params<'a>(
        lines: &'a [&'a str],
        config: &'a HashMap<String, serde_json::Value>,
    ) -> RuleParams<'a> {
        RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens: &[],
            config,
        }
    }

    #[test]
    fn test_md046_fenced_only() {
        let lines = vec!["# Title\n", "\n", "```\n", "code\n", "```\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 0, "Fenced-only should not trigger MD046");
    }

    #[test]
    fn test_md046_indented_only() {
        let lines = vec!["# Title\n", "\n", "    code block\n", "    more code\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 0, "Indented-only should not trigger MD046");
    }

    #[test]
    fn test_md046_mixed_styles_consistent() {
        let lines = vec![
            "# Title\n",
            "\n",
            "```\n",
            "fenced code\n",
            "```\n",
            "\n",
            "    indented code\n",
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        // Primary error + 0 helper deletes (single-line indented block)
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(
            main_errors.len(),
            1,
            "Mixed styles should report indented block"
        );
        assert_eq!(main_errors[0].line_number, 7);
        assert_eq!(
            main_errors[0].error_detail,
            Some("Expected: fenced; Actual: indented".to_string())
        );
    }

    #[test]
    fn test_md046_style_fenced() {
        // With style=fenced, indented blocks are errors even without fenced blocks
        let lines = vec!["# Title\n", "\n", "    indented code\n"];
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("fenced".to_string()),
        );
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(main_errors.len(), 1);
        assert_eq!(main_errors[0].line_number, 3);
    }

    #[test]
    fn test_md046_style_indented() {
        // With style=indented, fenced blocks are errors even without indented blocks
        let lines = vec!["# Title\n", "\n", "```\n", "code\n", "```\n"];
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("indented".to_string()),
        );
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(main_errors.len(), 1);
        assert_eq!(main_errors[0].line_number, 3);
    }

    #[test]
    fn test_md046_tilde_fenced() {
        let lines = vec!["~~~\n", "code\n", "~~~\n", "\n", "    indented\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(
            main_errors.len(),
            1,
            "Tilde fenced + indented should trigger mixed style error"
        );
        assert_eq!(main_errors[0].line_number, 5);
    }

    #[test]
    fn test_md046_no_code_blocks() {
        let lines = vec!["# Title\n", "\n", "Just a paragraph.\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 0, "No code blocks should not trigger MD046");
    }

    #[test]
    fn test_md046_indented_needs_blank_before() {
        // 4-space indent immediately after a paragraph is NOT an indented code block
        let lines = vec!["# Title\n", "Some text\n", "    not code\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md046_multiple_blocks() {
        let lines = vec![
            "# Title\n",
            "\n",
            "    indented 1\n",
            "\n",
            "paragraph\n",
            "\n",
            "    indented 2\n",
            "\n",
            "```\n",
            "fenced\n",
            "```\n",
        ];
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("fenced".to_string()),
        );
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(
            main_errors.len(),
            2,
            "Both indented blocks should be flagged"
        );
    }

    #[test]
    fn test_md046_has_fix_info() {
        let lines = vec!["```\n", "code\n", "```\n", "\n", "    indented\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(main_errors.len(), 1);
        assert!(
            main_errors[0].fix_info.is_some(),
            "MD046 should have fix_info"
        );
    }

    #[test]
    fn test_md046_fix_indented_to_fenced() {
        // Single indented block with style=fenced
        let lines = vec!["# Title\n", "\n", "    code line 1\n", "    code line 2\n"];
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("fenced".to_string()),
        );
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(main_errors.len(), 1);
        let fix = main_errors[0]
            .fix_info
            .as_ref()
            .expect("Should have fix_info");
        assert_eq!(fix.line_number, Some(3));
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(i32::MAX));
        // Replacement should be the fenced version
        let expected_text = "```\ncode line 1\ncode line 2\n```";
        assert_eq!(fix.insert_text, Some(expected_text.to_string()));
    }

    #[test]
    fn test_md046_fix_fenced_to_indented() {
        // Single fenced block with style=indented
        let lines = vec![
            "# Title\n",
            "\n",
            "```\n",
            "code line 1\n",
            "code line 2\n",
            "```\n",
        ];
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("indented".to_string()),
        );
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(main_errors.len(), 1);
        let fix = main_errors[0]
            .fix_info
            .as_ref()
            .expect("Should have fix_info");
        assert_eq!(fix.line_number, Some(3));
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(i32::MAX));
        // Replacement should be the indented version
        let expected_text = "    code line 1\n    code line 2";
        assert_eq!(fix.insert_text, Some(expected_text.to_string()));
    }

    #[test]
    fn test_md046_fix_helper_delete_lines() {
        // Fenced block with 2 content lines: opening + 2 content + closing = 4 lines
        // Primary fix on line 3, helpers delete lines 4, 5, 6
        let lines = vec!["# Title\n", "\n", "```\n", "line 1\n", "line 2\n", "```\n"];
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("indented".to_string()),
        );
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        // 1 primary + 3 helper deletes (lines 4, 5, 6)
        let helper_errors: Vec<_> = errors.iter().filter(|e| e.fix_only).collect();
        assert_eq!(helper_errors.len(), 3);
        for (i, helper) in helper_errors.iter().enumerate() {
            let fix = helper.fix_info.as_ref().unwrap();
            assert_eq!(fix.delete_count, Some(-1));
            assert_eq!(fix.line_number, Some(4 + i));
        }
    }

    #[test]
    fn test_md046_fix_empty_fenced_block() {
        // Empty fenced block -> indented
        let lines = vec!["# Title\n", "\n", "```\n", "```\n"];
        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            serde_json::Value::String("indented".to_string()),
        );
        let params = make_params(&lines, &config);
        let errors = MD046.lint(&params);
        let main_errors: Vec<_> = errors.iter().filter(|e| !e.fix_only).collect();
        assert_eq!(main_errors.len(), 1);
        let fix = main_errors[0]
            .fix_info
            .as_ref()
            .expect("Should have fix_info");
        assert_eq!(fix.insert_text, Some("    ".to_string()));
    }

    #[test]
    fn test_md046_unclosed_fence_no_panic() {
        // Unclosed fence at EOF should not panic
        let lines = vec!["# Title\n", "\n", "```rust\n", "code here\n"];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let _errors = MD046.lint(&params);
        // Should not panic, and unclosed fence is not counted as a complete block
    }
}
