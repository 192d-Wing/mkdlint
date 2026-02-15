//! MD053 - Link and image reference definitions should be needed

use crate::types::{LintError, ParserType, Rule, RuleParams};

pub struct MD053;

impl Rule for MD053 {
    fn names(&self) -> &[&'static str] {
        &["MD053", "link-image-reference-definitions"]
    }

    fn description(&self) -> &'static str {
        "Link and image reference definitions should be needed"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "images"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md053.md")
    }

    fn lint(&self, _params: &RuleParams) -> Vec<LintError> {
        // Requires reference tracking
        // Stub for now
        vec![]
    }
}
