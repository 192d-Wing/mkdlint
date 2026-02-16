//! MD003 - Heading style
//!
//! This rule checks that heading style is consistent throughout the document.
//! Supported styles:
//! - `atx`: ATX-style headings (e.g., `# Heading`)
//! - `atx_closed`: ATX-style headings with closing hashes (e.g., `# Heading #`)
//! - `setext`: Setext-style headings (underlined with `=` or `-`)
//! - `setext_with_atx`: Setext for h1 and h2, ATX for h3-h6
//! - `setext_with_atx_closed`: Setext for h1 and h2, ATX closed for h3-h6
//! - `consistent`: First heading determines the style

use crate::parser::TokenExt;
use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

#[cfg(test)]
use serde_json::Value;

pub struct MD003;

#[derive(Debug, PartialEq, Clone, Copy)]
enum HeadingStyle {
    Atx,
    AtxClosed,
    Setext,
}

impl HeadingStyle {
    fn as_str(&self) -> &'static str {
        match self {
            HeadingStyle::Atx => "atx",
            HeadingStyle::AtxClosed => "atx_closed",
            HeadingStyle::Setext => "setext",
        }
    }
}

/// Determines the heading style from the actual text
fn get_heading_style(lines: &[String], start_line: usize, end_line: usize) -> HeadingStyle {
    if start_line == 0 || start_line > lines.len() {
        return HeadingStyle::Atx;
    }

    let line_idx = start_line - 1;
    let line = &lines[line_idx];
    let trimmed = line.trim();

    // Check if it's an ATX-style heading (starts with #)
    if trimmed.starts_with('#') {
        // Check if it's closed (ends with #)
        let text_without_leading_hashes = trimmed.trim_start_matches('#').trim();
        if text_without_leading_hashes.ends_with('#') {
            // Make sure it's not just the content containing #
            let without_trailing = text_without_leading_hashes.trim_end_matches('#').trim_end();
            if without_trailing.len() < text_without_leading_hashes.len() - 1 {
                return HeadingStyle::AtxClosed;
            }
        }
        return HeadingStyle::Atx;
    }

    // Check if it's a Setext-style heading (underlined)
    // Setext headings span two lines: text line and underline line
    if end_line > start_line && end_line <= lines.len() {
        let underline_idx = end_line - 1;
        let underline = lines[underline_idx].trim();

        // Check if the next line is all = or all -
        if !underline.is_empty()
            && (underline.chars().all(|c| c == '=') || underline.chars().all(|c| c == '-'))
        {
            return HeadingStyle::Setext;
        }
    }

    HeadingStyle::Atx
}

/// Gets the heading level (1-6)
fn get_heading_level(lines: &[String], start_line: usize, end_line: usize) -> usize {
    if start_line == 0 || start_line > lines.len() {
        return 1;
    }

    let line_idx = start_line - 1;
    let line = &lines[line_idx];
    let trimmed = line.trim();

    // ATX style: count the # symbols
    if trimmed.starts_with('#') {
        let count = trimmed.chars().take_while(|&c| c == '#').count();
        return count.min(6);
    }

    // Setext style: = is h1, - is h2
    if end_line > start_line && end_line <= lines.len() {
        let underline_idx = end_line - 1;
        let underline = lines[underline_idx].trim();

        if !underline.is_empty() {
            if underline.chars().all(|c| c == '=') {
                return 1;
            } else if underline.chars().all(|c| c == '-') {
                return 2;
            }
        }
    }

    1
}

impl Rule for MD003 {
    fn names(&self) -> &[&'static str] {
        &["MD003", "heading-style"]
    }

    fn description(&self) -> &'static str {
        "Heading style"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md003.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Get configured style (default: "consistent")
        let configured_style = params
            .config
            .get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("consistent")
            .to_string();

        let headings = params.tokens.filter_by_type("heading");

        // Track the first heading style for "consistent" mode
        let mut first_style: Option<HeadingStyle> = None;

        for heading in headings {
            let style = get_heading_style(params.lines, heading.start_line, heading.end_line);
            let level = get_heading_level(params.lines, heading.start_line, heading.end_line);

            // For consistent mode, use the first heading's style
            if configured_style == "consistent" {
                if let Some(first) = first_style {
                    // Compare with first style
                    if style != first {
                        errors.push(LintError {
                            line_number: heading.start_line,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(format!(
                                "Expected: {}; Actual: {}",
                                first.as_str(),
                                style.as_str()
                            )),
                            error_context: None,
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: None,
                            fix_info: None,
                            severity: Severity::Error,
                        });
                    }
                } else {
                    // First heading - establish the style
                    first_style = Some(style);
                }
            } else {
                // Check against configured style
                let expected_style = match configured_style.as_str() {
                    "atx" => {
                        if style != HeadingStyle::Atx {
                            Some(("atx", style))
                        } else {
                            None
                        }
                    }
                    "atx_closed" => {
                        if style != HeadingStyle::AtxClosed {
                            Some(("atx_closed", style))
                        } else {
                            None
                        }
                    }
                    "setext" => {
                        if style != HeadingStyle::Setext {
                            Some(("setext", style))
                        } else {
                            None
                        }
                    }
                    "setext_with_atx" => {
                        // h1 and h2 should be setext, h3-h6 should be atx
                        if level <= 2 {
                            if style != HeadingStyle::Setext {
                                Some(("setext", style))
                            } else {
                                None
                            }
                        } else if style != HeadingStyle::Atx {
                            Some(("atx", style))
                        } else {
                            None
                        }
                    }
                    "setext_with_atx_closed" => {
                        // h1 and h2 should be setext, h3-h6 should be atx_closed
                        if level <= 2 {
                            if style != HeadingStyle::Setext {
                                Some(("setext", style))
                            } else {
                                None
                            }
                        } else if style != HeadingStyle::AtxClosed {
                            Some(("atx_closed", style))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some((expected, actual)) = expected_style {
                    errors.push(LintError {
                        line_number: heading.start_line,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!(
                            "Expected: {}; Actual: {}",
                            expected,
                            actual.as_str()
                        )),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
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
    use std::collections::HashMap;

    fn create_heading_token(start_line: usize, end_line: usize) -> Token {
        Token {
            token_type: "heading".to_string(),
            start_line,
            start_column: 1,
            end_line,
            end_column: 10,
            text: String::new(),
            children: vec![],
            parent: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_md003_consistent_all_atx() {
        let tokens = vec![
            create_heading_token(1, 1),
            create_heading_token(3, 3),
            create_heading_token(5, 5),
        ];

        let lines = vec![
            "# Heading 1\n".to_string(),
            "\n".to_string(),
            "## Heading 2\n".to_string(),
            "\n".to_string(),
            "### Heading 3\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), Value::String("consistent".to_string()));

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD003;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md003_consistent_mixed_styles() {
        let tokens = vec![create_heading_token(1, 1), create_heading_token(3, 4)];

        let lines = vec![
            "# Heading 1\n".to_string(),
            "\n".to_string(),
            "Heading 2\n".to_string(),
            "---------\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), Value::String("consistent".to_string()));

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD003;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
        assert!(errors[0].error_detail.as_ref().unwrap().contains("atx"));
        assert!(errors[0].error_detail.as_ref().unwrap().contains("setext"));
    }

    #[test]
    fn test_md003_atx_style() {
        let tokens = vec![create_heading_token(1, 1), create_heading_token(3, 4)];

        let lines = vec![
            "# Heading 1\n".to_string(),
            "\n".to_string(),
            "Heading 2\n".to_string(),
            "---------\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), Value::String("atx".to_string()));

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD003;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
    }

    #[test]
    fn test_md003_setext_style() {
        let tokens = vec![create_heading_token(1, 2), create_heading_token(4, 4)];

        let lines = vec![
            "Heading 1\n".to_string(),
            "=========\n".to_string(),
            "\n".to_string(),
            "# Heading 2\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), Value::String("setext".to_string()));

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD003;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 4);
    }

    #[test]
    fn test_md003_atx_closed_style() {
        let tokens = vec![create_heading_token(1, 1), create_heading_token(3, 3)];

        let lines = vec![
            "# Heading 1 #\n".to_string(),
            "\n".to_string(),
            "## Heading 2\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert("style".to_string(), Value::String("atx_closed".to_string()));

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD003;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
    }

    #[test]
    fn test_md003_setext_with_atx() {
        let tokens = vec![
            create_heading_token(1, 2),
            create_heading_token(4, 5),
            create_heading_token(7, 7),
        ];

        let lines = vec![
            "Heading 1\n".to_string(),
            "=========\n".to_string(),
            "\n".to_string(),
            "Heading 2\n".to_string(),
            "---------\n".to_string(),
            "\n".to_string(),
            "### Heading 3\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            Value::String("setext_with_atx".to_string()),
        );

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD003;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md003_setext_with_atx_closed() {
        let tokens = vec![create_heading_token(1, 2), create_heading_token(4, 4)];

        let lines = vec![
            "Heading 1\n".to_string(),
            "=========\n".to_string(),
            "\n".to_string(),
            "### Heading 3 ###\n".to_string(),
        ];

        let mut config = HashMap::new();
        config.insert(
            "style".to_string(),
            Value::String("setext_with_atx_closed".to_string()),
        );

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &config,
        };

        let rule = MD003;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_get_heading_style_atx() {
        let lines = vec!["# Heading\n".to_string()];
        assert_eq!(get_heading_style(&lines, 1, 1), HeadingStyle::Atx);
    }

    #[test]
    fn test_get_heading_style_atx_closed() {
        let lines = vec!["# Heading #\n".to_string()];
        assert_eq!(get_heading_style(&lines, 1, 1), HeadingStyle::AtxClosed);
    }

    #[test]
    fn test_get_heading_style_setext() {
        let lines = vec!["Heading\n".to_string(), "=======\n".to_string()];
        assert_eq!(get_heading_style(&lines, 1, 2), HeadingStyle::Setext);
    }

    #[test]
    fn test_get_heading_level_atx() {
        let lines = vec![
            "# H1\n".to_string(),
            "## H2\n".to_string(),
            "### H3\n".to_string(),
        ];
        assert_eq!(get_heading_level(&lines, 1, 1), 1);
        assert_eq!(get_heading_level(&lines, 2, 2), 2);
        assert_eq!(get_heading_level(&lines, 3, 3), 3);
    }

    #[test]
    fn test_get_heading_level_setext() {
        let lines = vec![
            "Heading 1\n".to_string(),
            "=========\n".to_string(),
            "Heading 2\n".to_string(),
            "---------\n".to_string(),
        ];
        assert_eq!(get_heading_level(&lines, 1, 2), 1);
        assert_eq!(get_heading_level(&lines, 3, 4), 2);
    }
}
