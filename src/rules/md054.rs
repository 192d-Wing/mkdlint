//! MD054 - Link and image style

use crate::types::{LintError, ParserType, Rule, RuleParams};

pub struct MD054;

impl Rule for MD054 {
    fn names(&self) -> &[&'static str] {
        &["MD054", "link-image-style"]
    }

    fn description(&self) -> &'static str {
        "Link and image style"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "images"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md054.md")
    }

    fn lint(&self, _params: &RuleParams) -> Vec<LintError> {
        // Style consistency check
        // Stub for now
        vec![]
    }
}
