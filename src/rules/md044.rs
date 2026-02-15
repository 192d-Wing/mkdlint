//! MD044 - Proper names should have the correct capitalization

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD044;

impl Rule for MD044 {
    fn names(&self) -> &[&'static str] {
        &["MD044", "proper-names"]
    }

    fn description(&self) -> &'static str {
        "Proper names should have the correct capitalization"
    }

    fn tags(&self) -> &[&'static str] {
        &["spelling"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md044.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        // Common proper names that are often misspelled
        let proper_names = [
            ("javascript", "JavaScript"),
            ("typescript", "TypeScript"),
            ("github", "GitHub"),
            ("nodejs", "Node.js"),
        ];

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let lower_line = line.to_lowercase();

            for (incorrect, correct) in &proper_names {
                if lower_line.contains(incorrect) && !line.contains(correct) {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!("Expected: {}; Actual: {}", correct, incorrect)),
                        error_context: None,
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
                        severity: Severity::Error,
                    });
                }
            }
        }

        errors
    }
}
