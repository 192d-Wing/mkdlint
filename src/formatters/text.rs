//! Plain text output formatter

use crate::types::LintResults;

/// Format lint results as plain text
pub fn format_text(results: &LintResults) -> String {
    results.to_string_with_alias(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LintError, Severity};

    #[test]
    fn test_format_text_empty() {
        let results = LintResults::new();
        assert_eq!(format_text(&results), "");
    }

    #[test]
    fn test_format_text_with_errors() {
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 1,
                rule_names: vec!["MD001".to_string(), "heading-increment".to_string()],
                rule_description: "Heading levels should increment by one".to_string(),
                severity: Severity::Error,
                ..Default::default()
            }],
        );
        let output = format_text(&results);
        assert!(output.contains("test.md"));
        assert!(output.contains("MD001"));
    }
}
