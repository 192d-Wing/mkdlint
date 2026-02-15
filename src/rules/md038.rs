//! MD038 - Spaces inside code span elements

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use once_cell::sync::Lazy;

static CODE_SPACE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"`( +[^`]+?[^ `]+ +)`").unwrap()
});

pub struct MD038;

impl Rule for MD038 {
    fn names(&self) -> &[&'static str] {
        &["MD038", "no-space-in-code"]
    }

    fn description(&self) -> &'static str {
        "Spaces inside code span elements"
    }

    fn tags(&self) -> &[&'static str] {
        &["whitespace", "code"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md038.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            for mat in CODE_SPACE_RE.find_iter(line) {
                errors.push(LintError {
                    line_number,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: None,
                    error_context: Some(mat.as_str().to_string()),
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: Some((mat.start() + 1, mat.len())),
                    fix_info: None,
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

    fn make_params<'a>(lines: &'a [String], tokens: &'a [crate::parser::Token], config: &'a HashMap<String, serde_json::Value>) -> crate::types::RuleParams<'a> {
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
    fn test_md038_no_spaces() {
        let lines: Vec<String> = "Use `code` here\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD038;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md038_with_spaces() {
        let lines: Vec<String> = "Use ` code ` here\n".lines().map(|l| l.to_string()).collect();
        let tokens = vec![];
        let config = HashMap::new();
        let params = make_params(&lines, &tokens, &config);
        let rule = MD038;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }
}
