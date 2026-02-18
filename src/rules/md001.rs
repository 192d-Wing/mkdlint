//! MD001 - Heading levels should only increment by one level at a time
//!
//! This rule checks that heading levels only increment by one at a time.
//! For example, an h3 heading should not appear directly after an h1 heading.

use crate::parser::{Token, TokenExt};
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;

pub struct MD001;

impl MD001 {
    /// Get the heading level from a heading token's metadata
    fn get_heading_level(heading: &Token) -> usize {
        heading
            .metadata
            .get("level")
            .and_then(|l| l.parse::<usize>().ok())
            .unwrap_or(1)
    }

    /// Check if front matter has a title field
    fn front_matter_has_title(
        front_matter_lines: &[&str],
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
    fn names(&self) -> &'static [&'static str] {
        &["MD001", "heading-increment"]
    }

    fn description(&self) -> &'static str {
        "Heading levels should only increment by one level at a time"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "fixable"]
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

        // Filter for heading tokens
        let headings = params.tokens.filter_by_type("heading");

        for heading in headings {
            let level = Self::get_heading_level(heading);

            // Only report error if level increases by more than 1
            if level > prev_level.saturating_add(1) {
                let expected_level = prev_level + 1;
                let is_setext = heading
                    .metadata
                    .get("setext")
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(false);

                // Generate fix_info to adjust the heading level
                let fix_info = if !is_setext {
                    // ATX heading: change the number of # characters
                    let line = params
                        .lines
                        .get(heading.start_line - 1)
                        .copied()
                        .unwrap_or("");

                    // Find where the heading text starts (after leading # and spaces)
                    if let Some(hash_count) = line.find(|c| c != '#' && c != ' ') {
                        let new_prefix = "#".repeat(expected_level) + " ";
                        Some(FixInfo {
                            line_number: Some(heading.start_line),
                            edit_column: Some(1),
                            delete_count: Some(hash_count as i32),
                            insert_text: Some(new_prefix),
                        })
                    } else {
                        None
                    }
                } else {
                    // Setext heading: convert to ATX format
                    // Replace both the heading line and the underline
                    let line = params
                        .lines
                        .get(heading.start_line - 1)
                        .map(|s| s.trim_end())
                        .unwrap_or("");

                    let new_heading = format!("{} {}", "#".repeat(expected_level), line);
                    Some(FixInfo {
                        line_number: Some(heading.start_line),
                        edit_column: Some(1),
                        delete_count: Some(i32::MAX), // Delete entire line (will be handled by apply_fixes)
                        insert_text: Some(new_heading),
                    })
                };

                errors.push(LintError {
                    line_number: heading.start_line,
                    rule_names: self.names(),
                    rule_description: self.description(),
                    error_detail: Some(format!(
                        "Expected: h{}; Actual: h{}",
                        expected_level, level
                    )),
                    error_context: None,
                    rule_information: self.information(),
                    error_range: None,
                    fix_info,
                    suggestion: Some(
                        "Heading levels should increment by one level at a time".to_string(),
                    ),
                    severity: Severity::Error,
                    fix_only: false,
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
    use std::collections::HashMap;

    /// Helper to create a heading token with metadata
    fn create_heading(line: usize, level: usize, setext: bool) -> Token {
        let mut metadata = HashMap::new();
        metadata.insert("level".to_string(), level.to_string());
        metadata.insert("setext".to_string(), setext.to_string());

        Token {
            token_type: "heading".to_string(),
            start_line: line,
            start_column: 1,
            end_line: if setext { line + 1 } else { line },
            end_column: 20,
            text: format!("Heading {}", level),
            children: vec![],
            parent: None,
            metadata,
        }
    }

    #[test]
    fn test_md001_valid_increment() {
        let tokens = vec![
            create_heading(1, 1, false),
            create_heading(2, 2, false),
            create_heading(3, 3, false),
        ];

        let lines = vec!["# Heading 1\n", "## Heading 2\n", "### Heading 3\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_skip_level() {
        let tokens = vec![
            create_heading(1, 1, false),
            create_heading(2, 3, false), // Skip from h1 to h3
        ];

        let lines = vec!["# Heading 1\n", "### Heading 3\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(
            errors[0].error_detail,
            Some("Expected: h2; Actual: h3".to_string())
        );
    }

    #[test]
    fn test_md001_decrease_is_ok() {
        let tokens = vec![
            create_heading(1, 1, false),
            create_heading(2, 2, false),
            create_heading(3, 3, false),
            create_heading(4, 1, false), // Back to h1 is ok
        ];

        let lines = vec![
            "# Heading 1\n",
            "## Heading 2\n",
            "### Heading 3\n",
            "# Heading 1 again\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_with_front_matter_title() {
        let tokens = vec![
            create_heading(4, 2, false), // h2 after front matter title is ok
        ];

        let lines = vec![
            "---\n",
            "title: Document Title\n",
            "---\n",
            "## Heading 2\n",
        ];

        let front_matter = vec!["title: Document Title\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &front_matter,
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_setext_headings() {
        let tokens = vec![create_heading(1, 1, true), create_heading(4, 2, true)];

        let lines = vec![
            "Heading 1\n",
            "=========\n",
            "\n",
            "Heading 2\n",
            "---------\n",
        ];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md001_multiple_skips() {
        let tokens = vec![
            create_heading(1, 1, false),
            create_heading(2, 4, false), // Skip from h1 to h4
            create_heading(3, 6, false), // Skip from h4 to h6
        ];

        let lines = vec!["# Heading 1\n", "#### Heading 4\n", "###### Heading 6\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line_number, 2);
        assert_eq!(errors[1].line_number, 3);
    }

    #[test]
    fn test_md001_fix_info_atx() {
        let tokens = vec![
            create_heading(1, 1, false),
            create_heading(2, 3, false), // Skip from h1 to h3
        ];

        let lines = vec!["# Heading 1\n", "### Heading 3\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);

        assert_eq!(errors.len(), 1);
        assert!(errors[0].fix_info.is_some());
        let fix = errors[0].fix_info.as_ref().unwrap();
        assert_eq!(fix.line_number, Some(2));
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.insert_text, Some("## ".to_string()));
    }

    #[test]
    fn test_md001_fix_info_setext() {
        let tokens = vec![
            create_heading(1, 1, false),
            create_heading(2, 2, true), // Setext h2 after h1 is ok, no error
        ];

        let lines = vec!["# Heading 1\n", "Heading 2\n", "---------\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
            workspace_headings: None,
        };

        let rule = MD001;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
