//! JSON output formatter

use crate::types::LintResults;

/// Format lint results as JSON
pub fn format_json(results: &LintResults) -> String {
    serde_json::to_string_pretty(results).unwrap_or_else(|e| {
        format!("{{\"error\": \"Failed to serialize results: {}\"}}", e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LintError, Severity};

    #[test]
    fn test_format_json_empty() {
        let results = LintResults::new();
        let output = format_json(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["results"].is_object());
    }

    #[test]
    fn test_format_json_with_errors() {
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 5,
                rule_names: vec!["MD009".to_string()],
                rule_description: "Trailing spaces".to_string(),
                error_detail: Some("Expected: 0; Actual: 3".to_string()),
                severity: Severity::Error,
                ..Default::default()
            }],
        );
        let output = format_json(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let errors = &parsed["results"]["test.md"];
        assert_eq!(errors[0]["line_number"], 5);
        assert_eq!(errors[0]["rule_names"][0], "MD009");
    }
}
