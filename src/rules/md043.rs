//! MD043 - Required heading structure

use crate::types::{LintError, ParserType, Rule, RuleParams};

pub struct MD043;

impl Rule for MD043 {
    fn names(&self) -> &[&'static str] {
        &["MD043", "required-headings", "required-headers"]
    }

    fn description(&self) -> &'static str {
        "Required heading structure"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md043.md")
    }

    fn lint(&self, _params: &RuleParams) -> Vec<LintError> {
        // This rule requires configuration to specify the required heading structure
        // For now, return empty as it needs custom config support
        vec![]
    }
}
