//! MD019 - Multiple spaces after hash on atx style heading

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD019;

impl Rule for MD019 {
    fn names(&self) -> &'static [&'static str] {
        &["MD019", "no-multiple-space-atx"]
    }

    fn description(&self) -> &'static str {
        "Multiple spaces after hash on atx style heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "atx", "spaces", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md019.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                if hash_count > 0 && hash_count <= 6 {
                    let after_hash = &trimmed[hash_count..];
                    let space_count = after_hash.chars().take_while(|&c| c == ' ').count();

                    if space_count > 1 {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!("Expected: 1; Actual: {}", space_count)),
                            error_context: None,
                            rule_information: self.information(),
                            error_range: Some((hash_count + 2, space_count - 1)),
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(hash_count + 2),
                                delete_count: Some((space_count - 1) as i32),
                                insert_text: None,
                            }),
                            suggestion: Some(
                                "Remove multiple spaces after hash on ATX heading".to_string(),
                            ),
                            severity: Severity::Error,
                            fix_only: false,
                        });
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
    fn test_md019_single_space() {
        let lines = vec!["# Heading\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD019.lint(&params).len(), 0);
    }

    #[test]
    fn test_md019_multiple_spaces() {
        let lines = vec!["#  Heading\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD019.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
    }

    #[test]
    fn test_md019_many_spaces_h2() {
        let lines = vec!["##   Heading 2\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD019.lint(&params).len(), 1);
    }

    #[test]
    fn test_md019_fix_info() {
        let lines = vec!["#   Title\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD019.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0].fix_info.as_ref().expect("fix_info");
        assert_eq!(fix.edit_column, Some(3)); // after "# ", delete extra spaces
        assert_eq!(fix.delete_count, Some(2)); // 3 spaces - 1 = 2 extra
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md019_all_heading_levels() {
        for level in 1..=6 {
            let hashes = "#".repeat(level);
            let line = format!("{}  Heading\n", hashes);
            let lines = vec![line.as_str()];
            let config = HashMap::new();
            let params = crate::types::RuleParams::test(&lines, &config);
            let errors = MD019.lint(&params);
            assert_eq!(errors.len(), 1, "level {} should trigger", level);
        }
    }

    #[test]
    fn test_md019_seven_hashes_ignored() {
        // ####### is not a valid heading (max 6)
        let lines = vec!["#######  Not a heading\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD019.lint(&params).len(), 0);
    }

    #[test]
    fn test_md019_no_space_no_error() {
        // No space at all (e.g. "#Heading") — not MD019's concern
        let lines = vec!["#Heading\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        assert_eq!(MD019.lint(&params).len(), 0);
    }

    #[test]
    fn test_md019_error_detail() {
        let lines = vec!["#    Title\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = MD019.lint(&params);
        assert_eq!(
            errors[0].error_detail.as_deref(),
            Some("Expected: 1; Actual: 4")
        );
    }
}
