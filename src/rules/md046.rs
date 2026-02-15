//! MD046 - Code block style

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD046;

impl Rule for MD046 {
    fn names(&self) -> &[&'static str] {
        &["MD046", "code-block-style"]
    }

    fn description(&self) -> &'static str {
        "Code block style"
    }

    fn tags(&self) -> &[&'static str] {
        &["code"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md046.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut fenced_count = 0;
        let mut indented_count = 0;

        for (idx, line) in params.lines.iter().enumerate() {
            let _line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                fenced_count += 1;
            } else if line.starts_with("    ") && !trimmed.is_empty() {
                // Potential indented code block
                indented_count += 1;
            }
        }

        // If both styles are used, report error
        if fenced_count > 0 && indented_count > 0 {
            errors.push(LintError {
                line_number: 1,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: Some("Mixed code block styles (fenced and indented)".to_string()),
                error_context: None,
                rule_information: self.information().map(|s| s.to_string()),
                error_range: None,
                fix_info: None,
                severity: Severity::Error,
            });
        }

        errors
    }
}
