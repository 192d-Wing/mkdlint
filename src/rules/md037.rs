//! MD037 - Spaces inside emphasis markers

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static EMPHASIS_SPACE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\*|_)( +[^*_]+?[^ *_]+ +)(\*|_)").unwrap());

pub struct MD037;

impl Rule for MD037 {
    fn names(&self) -> &[&'static str] {
        &["MD037", "no-space-in-emphasis"]
    }

    fn description(&self) -> &'static str {
        "Spaces inside emphasis markers"
    }

    fn tags(&self) -> &[&'static str] {
        &["whitespace", "emphasis", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md037.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            for caps in EMPHASIS_SPACE_RE.captures_iter(line) {
                let full_match = caps.get(0).unwrap();
                let open_marker = caps.get(1).unwrap().as_str();
                let inner_content = caps.get(2).unwrap().as_str();
                let close_marker = caps.get(3).unwrap().as_str();
                let trimmed = inner_content.trim();
                let corrected = format!("{}{}{}", open_marker, trimmed, close_marker);

                errors.push(LintError {
                    line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(full_match.as_str().to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((full_match.start() + 1, full_match.len())),
                    fix_info: Some(FixInfo {
                        line_number: None,
                        edit_column: Some(full_match.start() + 1),
                        delete_count: Some(full_match.len() as i32),
                        insert_text: Some(corrected),
                    }),
                    suggestion: Some("Remove spaces inside emphasis markers".to_string()),
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

    fn make_params<'a>(
        lines: &'a [String],
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
    fn test_md037_no_spaces() {
        let lines: Vec<String> = "This is *emphasis* text\n"
            .lines()
            .map(|l| l.to_string())
            .collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD037;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md037_with_spaces() {
        let lines: Vec<String> = "This is * emphasis * text\n"
            .lines()
            .map(|l| l.to_string())
            .collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD037;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md037_fix_info_asterisk() {
        // "This is * emphasis * text"
        //          ^-----------^ match at byte offset 8, length 12
        let lines: Vec<String> = vec!["This is * emphasis * text".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD037;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        // match starts at byte offset 8 => 1-based column 9
        assert_eq!(fix.edit_column, Some(9));
        // "* emphasis *" is 12 chars
        assert_eq!(fix.delete_count, Some(12));
        assert_eq!(fix.insert_text, Some("*emphasis*".to_string()));
    }

    #[test]
    fn test_md037_fix_info_underscore() {
        // "Some _ text _ here"
        //       ^-------^ match at byte offset 5, length 8
        let lines: Vec<String> = vec!["Some _ text _ here".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD037;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        assert_eq!(fix.edit_column, Some(6));
        assert_eq!(fix.delete_count, Some(8));
        assert_eq!(fix.insert_text, Some("_text_".to_string()));
    }

    #[test]
    fn test_md037_fix_info_no_error_no_fix() {
        let lines: Vec<String> = vec!["This is *emphasis* text".to_string()];
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD037;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
