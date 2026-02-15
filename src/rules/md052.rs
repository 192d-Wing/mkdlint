//! MD052 - Reference links and images should use a label that is defined

use crate::types::{LintError, ParserType, Rule, RuleParams};

pub struct MD052;

impl Rule for MD052 {
    fn names(&self) -> &[&'static str] {
        &["MD052", "reference-links-images"]
    }

    fn description(&self) -> &'static str {
        "Reference links and images should use a label that is defined"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "images"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md052.md")
    }

    fn lint(&self, _params: &RuleParams) -> Vec<LintError> {
        // Requires reference link validation
        // Stub for now
        vec![]
    }
}
