//! Plain text output formatter

use crate::types::{LintResults, Severity};
use colored::Colorize;
use std::collections::HashMap;

/// Format lint results as colored text with summary
pub fn format_text(results: &LintResults) -> String {
    format_text_with_context(results, &HashMap::new())
}

/// Format lint results with source context lines and error underlines
pub fn format_text_with_context(
    results: &LintResults,
    sources: &HashMap<String, String>,
) -> String {
    let mut output = Vec::new();
    let mut files: Vec<_> = results.results.keys().collect();
    files.sort();

    // Suppress emojis when color is disabled (--no-color, NO_COLOR env, or piped output)
    let use_emoji = colored::control::SHOULD_COLORIZE.should_colorize();

    for file in &files {
        if let Some(errors) = results.results.get(*file) {
            let source_lines: Option<Vec<&str>> = sources.get(*file).map(|s| s.lines().collect());

            for error in errors {
                if error.fix_only {
                    continue;
                }
                let rule_moniker = error.rule_names.join("/");

                let colored_rule = match error.severity {
                    Severity::Error => rule_moniker.red().to_string(),
                    Severity::Warning => rule_moniker.yellow().to_string(),
                };

                let mut line = format!(
                    "{}: {}: {} {}",
                    file.cyan(),
                    error.line_number.to_string().yellow(),
                    colored_rule,
                    error.rule_description
                );

                if let Some(detail) = &error.error_detail {
                    line.push_str(&format!(" {}", format!("[{}]", detail).dimmed()));
                }

                if let Some(context) = &error.error_context {
                    line.push_str(&format!(
                        " {}",
                        format!("[Context: \"{}\"]", context).dimmed()
                    ));
                }

                output.push(line);

                // Show suggestion if available
                if let Some(suggestion) = &error.suggestion {
                    let prefix = if use_emoji { "💡 " } else { "* " };
                    output.push(format!(
                        "  {}{}",
                        prefix.cyan(),
                        format!("Suggestion: {}", suggestion).cyan()
                    ));
                }

                // Show "fix available" indicator
                if error.fix_info.is_some() {
                    let prefix = if use_emoji { "🔧 " } else { "* " };
                    output.push(format!(
                        "  {}{}",
                        prefix.green(),
                        "Fix available - use --fix to apply automatically".green()
                    ));
                }

                // Show source line and underline if we have both source and error_range
                if let (Some(lines), Some((col_start, col_len))) =
                    (&source_lines, error.error_range)
                {
                    let line_idx = error.line_number.saturating_sub(1);
                    if line_idx < lines.len() {
                        let src = lines[line_idx];
                        let line_num_width = error.line_number.to_string().len();
                        let gutter = format!("{:>width$} |", "", width = line_num_width);
                        let numbered = format!(
                            "{:>width$} |  {}",
                            error.line_number,
                            src,
                            width = line_num_width
                        );
                        output.push(format!("  {}", gutter.dimmed()));
                        output.push(format!("  {}", numbered.dimmed()));

                        // Build underline: spaces up to col_start, then carets for col_len
                        let prefix_len = col_start.saturating_sub(1);
                        let caret_len = col_len.max(1);
                        let underline = format!(
                            "{:>width$} |  {}{}",
                            "",
                            " ".repeat(prefix_len),
                            "^".repeat(caret_len),
                            width = line_num_width,
                        );
                        let colored_underline = match error.severity {
                            Severity::Error => underline.red().to_string(),
                            Severity::Warning => underline.yellow().to_string(),
                        };
                        output.push(format!("  {}", colored_underline));
                    }
                }
            }
        }
    }

    // Summary line
    let error_count = results.error_count();
    let warning_count = results.warning_count();
    let file_count = results.files_with_errors().len();

    if error_count > 0 || warning_count > 0 {
        output.push(String::new());
        let summary = format!(
            "{} error(s), {} warning(s) in {} file(s)",
            error_count, warning_count, file_count
        );
        output.push(summary.bold().to_string());
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LintError;

    #[test]
    fn test_format_text_empty() {
        let results = LintResults::new();
        assert_eq!(format_text(&results), "");
    }

    #[test]
    fn test_format_text_with_errors() {
        colored::control::set_override(false);
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 1,
                rule_names: &["MD001", "heading-increment"],
                rule_description: "Heading levels should increment by one",
                severity: Severity::Error,
                fix_only: false,
                ..Default::default()
            }],
        );
        let output = format_text(&results);
        assert!(output.contains("test.md"));
        assert!(output.contains("MD001"));
    }

    #[test]
    fn test_format_text_summary() {
        colored::control::set_override(false);
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![
                LintError {
                    line_number: 1,
                    rule_names: &["MD001"],
                    rule_description: "test",
                    severity: Severity::Error,
                    fix_only: false,
                    ..Default::default()
                },
                LintError {
                    line_number: 2,
                    rule_names: &["MD059"],
                    rule_description: "test",
                    severity: Severity::Warning,
                    fix_only: false,
                    ..Default::default()
                },
            ],
        );
        let output = format_text(&results);
        assert!(output.contains("1 error(s), 1 warning(s) in 1 file(s)"));
    }

    #[test]
    fn test_format_text_with_source_context() {
        colored::control::set_override(false);
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 3,
                rule_names: &["MD009"],
                rule_description: "Trailing spaces",
                error_range: Some((12, 3)),
                severity: Severity::Error,
                fix_only: false,
                ..Default::default()
            }],
        );

        let mut sources = HashMap::new();
        sources.insert(
            "test.md".to_string(),
            "# Title\n\nSome text   \n".to_string(),
        );

        let output = format_text_with_context(&results, &sources);
        assert!(output.contains("Some text   "), "Should show source line");
        assert!(output.contains("^^^"), "Should show underline carets");
    }

    #[test]
    fn test_format_text_no_context_without_sources() {
        colored::control::set_override(false);
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 1,
                rule_names: &["MD009"],
                rule_description: "Trailing spaces",
                error_range: Some((5, 3)),
                severity: Severity::Error,
                fix_only: false,
                ..Default::default()
            }],
        );

        // No sources provided — should fall back to no underline
        let output = format_text(&results);
        assert!(!output.contains("^^^"), "No context without sources");
    }

    #[test]
    fn test_format_text_no_context_without_error_range() {
        colored::control::set_override(false);
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 1,
                rule_names: &["MD022"],
                rule_description: "Headings should be surrounded by blank lines",
                error_range: None,
                severity: Severity::Error,
                fix_only: false,
                ..Default::default()
            }],
        );

        let mut sources = HashMap::new();
        sources.insert("test.md".to_string(), "# Title\nSome text\n".to_string());

        let output = format_text_with_context(&results, &sources);
        // Has the error line but no underline (no error_range)
        assert!(!output.contains("^^^"), "No carets without error_range");
    }
}
