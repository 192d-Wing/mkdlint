//! MD023 - Headings must start at the beginning of the line

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD023;

impl Rule for MD023 {
    fn names(&self) -> &'static [&'static str] {
        &["MD023", "heading-start-left"]
    }

    fn description(&self) -> &'static str {
        "Headings must start at the beginning of the line"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "spaces", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md023.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            // Check if line starts with spaces/tabs followed by #
            if line.starts_with(' ') || line.starts_with('\t') {
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    let indent_count = line.len() - trimmed.len();
                    // Strip line endings for cross-platform compatibility
                    let trimmed_no_newline = trimmed.trim_end_matches('\n').trim_end_matches('\r');
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!("Expected: 0; Actual: {}", indent_count)),
                        error_context: Some({
                            let char_limit = trimmed_no_newline
                                .char_indices()
                                .nth(20)
                                .map(|(i, _)| i)
                                .unwrap_or(trimmed_no_newline.len());
                            trimmed_no_newline[..char_limit].to_string()
                        }),
                        rule_information: self.information(),
                        error_range: Some((1, indent_count)),
                        fix_info: Some(FixInfo {
                            line_number: None,
                            edit_column: Some(1),
                            delete_count: Some(indent_count as i32),
                            insert_text: None,
                        }),
                        suggestion: Some(
                            "Headings must start at the beginning of the line".to_string(),
                        ),
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
    use std::collections::HashMap;

    fn make_params<'a>(
        lines: &'a [&'a str],
        tokens: &'a [crate::parser::Token],
        config: &'a HashMap<String, serde_json::Value>,
    ) -> crate::types::RuleParams<'a> {
        crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens,
            config,
        }
    }

    #[test]
    fn test_md023_no_indent() {
        let lines: Vec<&str> = "# Heading\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD023;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md023_space_indent() {
        let lines: Vec<&str> = "  # Heading\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD023;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md023_tab_indent() {
        let lines: Vec<&str> = "\t# Heading\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD023;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md023_fix_info_spaces() {
        let lines: Vec<&str> = "  # Heading\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD023;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(2)); // 2 leading spaces
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md023_fix_info_tab() {
        let lines: Vec<&str> = "\t# Heading\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD023;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(1)); // 1 leading tab
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md023_fix_info_mixed_indent() {
        let lines: Vec<&str> = " \t  # Heading\n".lines().collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD023;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(4)); // 4 leading whitespace chars
        assert_eq!(fix.insert_text, None);
    }
}
