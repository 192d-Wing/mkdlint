//! MD025 - Multiple top-level headings in the same document

use crate::parser::TokenExt;
use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD025;

impl Rule for MD025 {
    fn names(&self) -> &[&'static str] {
        &["MD025", "single-title", "single-h1"]
    }

    fn description(&self) -> &'static str {
        "Multiple top-level headings in the same document"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md025.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let headings = params.tokens.filter_by_type("heading");
        let mut found_h1 = false;

        for heading in headings {
            // Check if it's an H1 via metadata
            let level = heading
                .metadata
                .get("level")
                .and_then(|l| l.parse::<u8>().ok())
                .unwrap_or(0);

            if level == 1 {
                if found_h1 {
                    // Generate fix to convert H1 to H2
                    let line = params.lines.get(heading.start_line - 1);
                    let fix_info = if let Some(line_text) = line {
                        let trimmed = line_text.trim_start();
                        if trimmed.starts_with('#') {
                            // ATX style heading - add one more #
                            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                            Some(FixInfo {
                                line_number: Some(heading.start_line),
                                edit_column: Some(1),
                                delete_count: Some(hash_count as i32),
                                insert_text: Some("##".to_string()),
                            })
                        } else {
                            // Setext style - convert to ATX H2
                            let heading_text = trimmed.trim_end();
                            Some(FixInfo {
                                line_number: Some(heading.start_line),
                                edit_column: Some(1),
                                delete_count: Some(i32::MAX),
                                insert_text: Some(format!("## {}", heading_text)),
                            })
                        }
                    } else {
                        None
                    };

                    errors.push(LintError {
                        line_number: heading.start_line,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: None,
                        error_context: Some(heading.text.trim().to_string()),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info,
                        suggestion: Some(
                            "Convert this heading to H2 (##) or restructure your document to have only one H1".to_string(),
                        ),
                        severity: Severity::Error,
                    });
                }
                found_h1 = true;
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

    fn make_heading(line: usize, text: &str, level: u8) -> Token {
        let mut t = Token::new("heading");
        t.start_line = line;
        t.end_line = line;
        t.text = text.to_string();
        t.metadata.insert("level".to_string(), level.to_string());
        t
    }

    #[test]
    fn test_md025_single_h1() {
        let tokens = vec![make_heading(1, "Title", 1), make_heading(3, "Section", 2)];
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "## Section\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD025.lint(&params);
        assert_eq!(errors.len(), 0, "Single H1 should not trigger MD025");
    }

    #[test]
    fn test_md025_multiple_h1() {
        let tokens = vec![
            make_heading(1, "Title", 1),
            make_heading(3, "Another Title", 1),
        ];
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "# Another Title\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD025.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 3);
        assert_eq!(errors[0].error_context, Some("Another Title".to_string()));
    }

    #[test]
    fn test_md025_three_h1() {
        let tokens = vec![
            make_heading(1, "First", 1),
            make_heading(3, "Second", 1),
            make_heading(5, "Third", 1),
        ];
        let lines = vec![
            "# First\n".to_string(),
            "\n".to_string(),
            "# Second\n".to_string(),
            "\n".to_string(),
            "# Third\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD025.lint(&params);
        assert_eq!(errors.len(), 2, "Second and third H1 should both error");
    }

    #[test]
    fn test_md025_no_h1() {
        let tokens = vec![
            make_heading(1, "Section", 2),
            make_heading(3, "Subsection", 3),
        ];
        let lines = vec![
            "## Section\n".to_string(),
            "\n".to_string(),
            "### Subsection\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD025.lint(&params);
        assert_eq!(errors.len(), 0, "No H1 headings should not trigger MD025");
    }

    #[test]
    fn test_md025_no_fix_info() {
        let tokens = vec![make_heading(1, "Title", 1), make_heading(3, "Second", 1)];
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "# Second\n".to_string(),
        ];
        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &HashMap::new(),
        };

        let errors = MD025.lint(&params);
        assert!(
            errors[0].fix_info.is_some(),
            "MD025 should have fix_info to convert H1 to H2"
        );
    }
}
