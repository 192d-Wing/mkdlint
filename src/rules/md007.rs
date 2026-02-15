//! MD007 - Unordered list indentation
//!
//! This rule checks that unordered list items have consistent indentation.

use crate::types::{LintError, ParserType, Rule, RuleParams};

pub struct MD007;

impl Rule for MD007 {
    fn names(&self) -> &[&'static str] {
        &["MD007", "ul-indent"]
    }

    fn description(&self) -> &'static str {
        "Unordered list indentation"
    }

    fn tags(&self) -> &[&'static str] {
        &["bullet", "ul", "indentation"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md007.md")
    }

    fn lint(&self, _params: &RuleParams) -> Vec<LintError> {
        // Simplified stub - full implementation was lost
        // TODO: Reimplement based on original markdownlint logic
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // TODO: Reimplement MD007
    fn test_md007_stub() {
        // Placeholder
    }
}
