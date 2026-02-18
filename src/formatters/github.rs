//! GitHub Actions workflow command formatter
//!
//! Outputs lint errors as GitHub Actions annotation commands:
//! `::error file={file},line={line},col={col},endLine={line},endColumn={endCol},title={rule}::{message}`
//!
//! These are picked up by GitHub Actions runners and displayed as PR annotations
//! in the Files Changed view.

use crate::types::{LintResults, Severity};

/// Format lint results as GitHub Actions workflow annotation commands.
///
/// Each error produces one line on stdout in the format:
/// ```text
/// ::error file=foo.md,line=5,col=1,endLine=5,endColumn=20,title=MD009::Trailing spaces [Expected: 0; Actual: 3]
/// ```
///
/// `fix_only` errors (internal auto-fix helpers) are silently skipped.
pub fn format_github(results: &LintResults) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut files: Vec<_> = results.results.keys().collect();
    files.sort();

    for file in &files {
        if let Some(errors) = results.results.get(*file) {
            for error in errors {
                if error.fix_only {
                    continue;
                }

                let level = match error.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                };

                let line = error.line_number;
                let (col, end_col) = match error.error_range {
                    Some((start_col, length)) => (start_col, start_col + length),
                    None => (1, 1),
                };

                let title = error.rule_names.first().copied().unwrap_or("mkdlint");

                let mut message = error.rule_description.to_string();
                if let Some(detail) = &error.error_detail {
                    message.push_str(&format!(" [{}]", detail));
                }

                lines.push(format!(
                    "::{level} file={file},line={line},col={col},endLine={line},endColumn={end_col},title={title}::{message}",
                ));
            }
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LintError, LintResults, Severity};

    fn make_error(severity: Severity, fix_only: bool) -> LintError {
        LintError {
            line_number: 5,
            rule_names: &["MD009", "no-trailing-spaces"],
            rule_description: "Trailing spaces",
            error_detail: Some("Expected: 0; Actual: 3".to_string()),
            error_range: Some((3, 10)),
            severity,
            fix_only,
            ..Default::default()
        }
    }

    #[test]
    fn test_format_github_error() {
        let mut results = LintResults::new();
        results.add(
            "foo.md".to_string(),
            vec![make_error(Severity::Error, false)],
        );
        let output = format_github(&results);
        assert!(
            output.starts_with("::error "),
            "Should start with ::error. Got: {output}"
        );
        assert!(output.contains("file=foo.md"), "Should include filename");
        assert!(output.contains("line=5"), "Should include line number");
        assert!(output.contains("title=MD009"), "Should include rule name");
        assert!(
            output.contains("Trailing spaces"),
            "Should include description"
        );
        assert!(
            output.contains("Expected: 0; Actual: 3"),
            "Should include detail"
        );
    }

    #[test]
    fn test_format_github_warning() {
        let mut results = LintResults::new();
        results.add(
            "bar.md".to_string(),
            vec![make_error(Severity::Warning, false)],
        );
        let output = format_github(&results);
        assert!(
            output.starts_with("::warning "),
            "Should start with ::warning. Got: {output}"
        );
    }

    #[test]
    fn test_format_github_skips_fix_only() {
        let mut results = LintResults::new();
        results.add(
            "baz.md".to_string(),
            vec![make_error(Severity::Error, true)],
        );
        let output = format_github(&results);
        assert!(output.is_empty(), "fix_only errors should be skipped");
    }

    #[test]
    fn test_format_github_column_range() {
        let mut results = LintResults::new();
        results.add(
            "foo.md".to_string(),
            vec![make_error(Severity::Error, false)],
        );
        let output = format_github(&results);
        // col=3, endColumn=13 (3+10)
        assert!(output.contains("col=3"), "Should include col");
        assert!(output.contains("endColumn=13"), "Should include endColumn");
    }
}
