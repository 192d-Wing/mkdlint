//! Convert mkdlint errors to LSP diagnostics

use crate::types::{LintError, Severity};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

use super::utils::{to_position, to_range};

/// Convert a LintError to an LSP Diagnostic
pub fn lint_error_to_diagnostic(error: &LintError, lines: &[String]) -> Diagnostic {
    let range = calculate_range(error, lines);
    let severity = severity_to_lsp(error.severity);
    let message = format_message(error);
    let source = Some("mkdlint".to_string());
    let code = error
        .rule_names
        .first()
        .map(|name| NumberOrString::String(name.to_string()));

    Diagnostic {
        range,
        severity: Some(severity),
        code,
        source,
        message,
        ..Default::default()
    }
}

/// Calculate the LSP Range for an error
fn calculate_range(error: &LintError, lines: &[String]) -> Range {
    if let Some((start_col, length)) = error.error_range {
        // Use error_range if available
        to_range(error.line_number, start_col, length)
    } else {
        // Fall back to highlighting the entire line
        let line_idx = error.line_number.saturating_sub(1);
        let line_content = lines.get(line_idx).map(|s| s.as_str()).unwrap_or("");

        // Trim trailing newline/whitespace for better UX
        let trimmed_len = line_content.trim_end().len();
        let start = to_position(error.line_number, 1);
        let end = Position {
            line: start.line,
            character: trimmed_len as u32,
        };

        Range { start, end }
    }
}

/// Convert mkdlint Severity to LSP DiagnosticSeverity
fn severity_to_lsp(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
    }
}

/// Format the diagnostic message
fn format_message(error: &LintError) -> String {
    let mut parts = vec![error.rule_description.to_string()];

    if let Some(detail) = &error.error_detail {
        parts.push(format!("({})", detail));
    }

    if let Some(context) = &error.error_context {
        parts.push(format!("[Context: \"{}\"]", context));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_error(
        line: usize,
        error_range: Option<(usize, usize)>,
        severity: Severity,
    ) -> LintError {
        LintError {
            line_number: line,
            rule_names: &["MD001"],
            rule_description: "Test rule",
            error_detail: Some("Detail".to_string()),
            error_context: Some("Context".to_string()),
            rule_information: None,
            error_range,
            fix_info: None,
            suggestion: Some("Fix this issue".to_string()),
            severity,
        }
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(severity_to_lsp(Severity::Error), DiagnosticSeverity::ERROR);
        assert_eq!(
            severity_to_lsp(Severity::Warning),
            DiagnosticSeverity::WARNING
        );
    }

    #[test]
    fn test_diagnostic_with_error_range() {
        let error = create_test_error(1, Some((5, 10)), Severity::Error);
        let lines = vec!["# Test heading\n".to_string()];
        let diagnostic = lint_error_to_diagnostic(&error, &lines);

        assert_eq!(diagnostic.range.start, Position::new(0, 4));
        assert_eq!(diagnostic.range.end, Position::new(0, 14));
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostic.source, Some("mkdlint".to_string()));
    }

    #[test]
    fn test_diagnostic_without_error_range() {
        let error = create_test_error(1, None, Severity::Warning);
        let lines = vec!["# Test heading\n".to_string()];
        let diagnostic = lint_error_to_diagnostic(&error, &lines);

        assert_eq!(diagnostic.range.start, Position::new(0, 0));
        // Should use trimmed line length
        assert_eq!(diagnostic.range.end.character, 14);
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn test_message_formatting() {
        let error = create_test_error(1, None, Severity::Error);
        let message = format_message(&error);
        assert_eq!(message, "Test rule (Detail) [Context: \"Context\"]");
    }

    #[test]
    fn test_message_no_context() {
        let mut error = create_test_error(1, None, Severity::Error);
        error.error_context = None;
        let message = format_message(&error);
        assert_eq!(message, "Test rule (Detail)");
    }

    #[test]
    fn test_diagnostic_code() {
        let error = create_test_error(1, None, Severity::Error);
        let lines = vec!["# Test\n".to_string()];
        let diagnostic = lint_error_to_diagnostic(&error, &lines);

        assert_eq!(
            diagnostic.code,
            Some(lsp_types::NumberOrString::String("MD001".to_string()))
        );
    }
}
