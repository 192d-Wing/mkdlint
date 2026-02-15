//! MD059 - Emphasis marker style in math

use crate::types::{LintError, ParserType, Rule, RuleParams};

pub struct MD059;

impl Rule for MD059 {
    fn names(&self) -> &[&'static str] {
        &["MD059", "emphasis-marker-style-math"]
    }

    fn description(&self) -> &'static str {
        "Emphasis marker style should not conflict with math syntax"
    }

    fn tags(&self) -> &[&'static str] {
        &["emphasis", "math"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md059.md")
    }

    fn lint(&self, _params: &RuleParams) -> Vec<LintError> {
        // Math-specific rule
        // Stub for now
        vec![]
    }
}
