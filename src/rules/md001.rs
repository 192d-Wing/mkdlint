//! MD001 - Heading levels should only increment by one level at a time
//!
//! This rule checks that heading levels only increment by one at a time.
//! For example, an h3 heading should not appear directly after an h1 heading.

use crate::parser::{Token, TokenExt};
use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;

pub struct MD001;

impl MD001 {
    /// Get the heading level from a heading token
    fn get_heading_level(heading: &Token, all_tokens: &[Token]) -> usize {
        // Look for atxHeadingSequence or setextHeadingLine in children
        for &child_idx in &heading.children {
            if let Some(child) = all_tokens.get(child_idx) {
                if child.token_type == "atxHeadingSequence" {
                    // Count the number of # characters
                    let hash_count = child.text.chars().filter(|&c| c == '#').count();
                    return hash_count.min(6);
                } else if child.token_type == "setextHeadingLine" {
                    // Check if it's = (level 1) or - (level 2)
                    if child.text.starts_with('=') {
                        return 1;
                    } else if child.text.starts_with('-') {
                        return 2;
                    }
                }
            }
        }
        1 // Default to level 1 if we can't determine
    }

    /// Check if front matter has a title field
    fn front_matter_has_title(
        front_matter_lines: &[String],
        config: &std::collections::HashMap<String, serde_json::Value>,
    ) -> bool {
        // Get front_matter_title pattern from config, default to checking for "title:"
        let pattern = match config.get("front_matter_title") {
            Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
            Some(serde_json::Value::Bool(false)) => return false, // Ignore front matter
            _ => r#"^\s*"?title"?\s*[:=]"#.to_string(),
        };

        let re = match Regex::new(&format!("(?i){}", pattern)) {
            Ok(r) => r,
            Err(_) => return false,
        };

        front_matter_lines.iter().any(|line| re.is_match(line))
    }
}

impl Rule for MD001 {
    fn names(&self) -> &[&'static str] {
        &["MD001", "heading-increment"]
    }

    fn description(&self) -> &'static str {
        "Heading levels should only increment by one level at a time"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md001.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Check if front matter has a title (acts as implicit h1)
        let has_title = Self::front_matter_has_title(params.front_matter_lines, params.config);
        let mut prev_level = if has_title {
            1
        } else {
            usize::MAX // Start with max so first heading is always valid
        };

        // Filter for heading tokens (both ATX and Setext)
        let headings = params
            .tokens
            .filter_by_types(&["atxHeading", "setextHeading"]);

        for heading in headings {
            let level = Self::get_heading_level(heading, params.tokens);

            // Only report error if level increases by more than 1
            if level > prev_level.saturating_add(1) {
                errors.push(LintError {
                    line_number: heading.start_line,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: Some(format!("Expected: h{}; Actual: h{}", prev_level + 1, level)),
                    error_context: None,
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: None,
                    fix_info: None,
                    severity: Severity::Error,
                });
            }

            prev_level = level;
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Token;
    use std::collections::HashMap;

    /// Helper to create an ATX heading token with children
    fn create_atx_heading(line: usize, level: usize, tokens: &mut Vec<Token>) -> Token {
        let hash_text = "#".repeat(level);
        let sequence_idx = tokens.len();

        // Create the heading sequence token
        tokens.push(Token {
            token_type: "atxHeadingSequence".to_string(),
            start_line: line,
            start_column: 1,
            end_line: line,
            end_column: level + 1,
            text: hash_text,
            children: vec![],
            parent: None,
        });

        // Create the heading token
        Token {
            token_type: "atxHeading".to_string(),
            start_line: line,
            start_column: 1,
            end_line: line,
            end_column: 20,
            text: format!("{} Heading", "#".repeat(level)),
            children: vec![sequence_idx],
            parent: None,
        }
    }

    /// Helper to create a setext heading token with children
    fn create_setext_heading(line: usize, level: usize, tokens: &mut Vec<Token>) -> Token {
        let underline_char = if level == 1 { '=' } else { '-' };
        let sequence_idx = tokens.len();

        // Create the heading line token
        tokens.push(Token {
            token_type: "setextHeadingLine".to_string(),
            start_line: line + 1,
            start_column: 1,
            end_line: line + 1,
            end_column: 5,
            text: underline_char.to_string().repeat(4),
            children: vec![],
            parent: None,
        });

        // Create the heading token
        Token {
            token_type: "setextHeading".to_string(),
            start_line: line,
            start_column: 1,
            end_line: line + 1,
            end_column: 5,
            text: "Heading".to_string(),
            children: vec![sequence_idx],
            parent: None,
        }
    }

    #[test]
    fn test_md001_valid_increment() {
        let mut tokens = Vec::new();
        let h1 = create_atx_heading(1, 1, &mut tokens);
        tokens.push(h1);
        let h2 = create_atx_heading(2, 2, &mut tokens);
        tokens.push(h2);
        let h3 = create_atx_heading(3, 3, &mut tokens);
        tokens.push(h3);

        let lines = vec![
            "# Heading 1\n".to_string(),
            "## Heading 2\n".to_string(),
            "### Heading 3\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_skip_level() {
        let mut tokens = Vec::new();
        let h1 = create_atx_heading(1, 1, &mut tokens);
        tokens.push(h1);
        let h3 = create_atx_heading(2, 3, &mut tokens); // Skip from h1 to h3
        tokens.push(h3);

        let lines = vec![
            "# Heading 1\n".to_string(),
            "### Heading 3\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD001;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(errors[0].error_detail, Some("Expected: h2; Actual: h3".to_string()));
    }

    #[test]
    fn test_md001_decrease_is_ok() {
        let mut tokens = Vec::new();
        let h1 = create_atx_heading(1, 1, &mut tokens);
        tokens.push(h1);
        let h2 = create_atx_heading(2, 2, &mut tokens);
        tokens.push(h2);
        let h3 = create_atx_heading(3, 3, &mut tokens);
        tokens.push(h3);
        let h1_again = create_atx_heading(4, 1, &mut tokens); // Back to h1 is ok
        tokens.push(h1_again);

        let lines = vec![
            "# Heading 1\n".to_string(),
            "## Heading 2\n".to_string(),
            "### Heading 3\n".to_string(),
            "# Heading 1 again\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_with_front_matter_title() {
        let mut tokens = Vec::new();
        let h2 = create_atx_heading(4, 2, &mut tokens); // h2 after front matter title is ok
        tokens.push(h2);

        let lines = vec![
            "---\n".to_string(),
            "title: Document Title\n".to_string(),
            "---\n".to_string(),
            "## Heading 2\n".to_string(),
        ];

        let front_matter = vec![
            "title: Document Title\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &front_matter,
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_setext_headings() {
        let mut tokens = Vec::new();
        let h1 = create_setext_heading(1, 1, &mut tokens); // h1
        tokens.push(h1);
        let h2 = create_setext_heading(4, 2, &mut tokens); // h2
        tokens.push(h2);

        let lines = vec![
            "Heading 1\n".to_string(),
            "=========\n".to_string(),
            "\n".to_string(),
            "Heading 2\n".to_string(),
            "---------\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_multiple_skips() {
        let mut tokens = Vec::new();
        let h1 = create_atx_heading(1, 1, &mut tokens);
        tokens.push(h1);
        let h4 = create_atx_heading(2, 4, &mut tokens); // Skip from h1 to h4
        tokens.push(h4);
        let h6 = create_atx_heading(3, 6, &mut tokens); // Skip from h4 to h6
        tokens.push(h6);

        let lines = vec![
            "# Heading 1\n".to_string(),
            "#### Heading 4\n".to_string(),
            "###### Heading 6\n".to_string(),
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let rule = MD001;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(errors[1].line_number, 3);
    }
}
