//! MD011 - Reversed link syntax
//!
//! This rule checks for reversed link syntax like (text)[link] instead of [text](link)

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static REVERSED_LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\(([^)]+)\)\[([^\]]+)\]").unwrap());

pub struct MD011;

impl Rule for MD011 {
    fn names(&self) -> &[&'static str] {
        &["MD011", "no-reversed-links"]
    }

    fn description(&self) -> &'static str {
        "Reversed link syntax"
    }

    fn tags(&self) -> &[&'static str] {
        &["links"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md011.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            for caps in REVERSED_LINK_RE.captures_iter(line) {
                let mat = caps.get(0).unwrap();
                let text = caps.get(1).unwrap().as_str();
                let url = caps.get(2).unwrap().as_str();
                let corrected = format!("[{}]({})", text, url);

                errors.push(LintError {
                    line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(mat.as_str().to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((mat.start() + 1, mat.len())),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(mat.start() + 1),
                        delete_count: Some(mat.len() as i32),
                        insert_text: Some(corrected),
                    }),
                    severity: Severity::Error,
                });
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
    fn test_md011_correct_syntax() {
        let lines = vec!["[text](link)\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD011;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md011_reversed_syntax() {
        let lines = vec!["(text)[link]\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD011;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md011_fix_info() {
        let lines = vec!["(text)[link]\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD011;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(1));
        assert_eq!(fix.delete_count, Some(12)); // "(text)[link]" is 12 chars
        assert_eq!(fix.insert_text, Some("[text](link)".to_string()));
    }

    #[test]
    fn test_md011_fix_info_with_offset() {
        let lines = vec!["See (hello)[world] for details\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD011;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(5)); // "(hello)" starts at column 5 (1-based)
        assert_eq!(fix.delete_count, Some(14)); // "(hello)[world]" is 14 chars
        assert_eq!(fix.insert_text, Some("[hello](world)".to_string()));
    }
}
