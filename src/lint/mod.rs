//! Core linting functionality

use crate::config::Config;
use crate::parser;
use crate::types::{LintError, LintOptions, LintResults, MarkdownlintError, Result};
use rayon::prelude::*;

/// Lint markdown content synchronously
///
/// Files are read sequentially (for proper error reporting) then linted
/// in parallel using rayon.
pub fn lint_sync(options: &LintOptions) -> Result<LintResults> {
    let mut results = LintResults::new();

    // Load configuration
    let config = load_config(options)?;

    // Read all files first (sequential for proper error reporting)
    let mut inputs: Vec<(String, String)> = Vec::new();
    for file_path in &options.files {
        let content = std::fs::read_to_string(file_path)
            .map_err(|_| MarkdownlintError::FileNotFound(file_path.clone()))?;
        inputs.push((file_path.clone(), content));
    }
    for (name, content) in &options.strings {
        inputs.push((name.clone(), content.clone()));
    }

    // Lint all inputs in parallel
    let file_results: Vec<(String, std::result::Result<Vec<LintError>, MarkdownlintError>)> =
        inputs
            .par_iter()
            .map(|(name, content)| {
                let errors = lint_content(content, &config, name);
                (name.clone(), errors)
            })
            .collect();

    for (name, result) in file_results {
        results.add(name, result?);
    }

    Ok(results)
}

/// Lint markdown content asynchronously
///
/// Files are read concurrently with tokio, then linted in parallel
/// using spawn_blocking (CPU-bound work).
#[cfg(feature = "async")]
pub async fn lint_async(options: &LintOptions) -> Result<LintResults> {
    use std::sync::Arc;
    use tokio::fs;

    let mut results = LintResults::new();

    // Load configuration
    let config = Arc::new(load_config(options)?);

    // Read all files concurrently
    let read_handles: Vec<_> = options
        .files
        .iter()
        .map(|file_path| {
            let path = file_path.clone();
            tokio::spawn(async move {
                let content = fs::read_to_string(&path)
                    .await
                    .map_err(|_| MarkdownlintError::FileNotFound(path.clone()));
                (path, content)
            })
        })
        .collect();

    let mut inputs: Vec<(String, String)> = Vec::new();
    for handle in read_handles {
        let (path, content_result) = handle.await.map_err(|e| {
            MarkdownlintError::AsyncRuntime(format!("Task join error: {}", e))
        })?;
        inputs.push((path, content_result?));
    }

    // Add string inputs
    for (name, content) in &options.strings {
        inputs.push((name.clone(), content.clone()));
    }

    // Lint all inputs concurrently using spawn_blocking (CPU-bound)
    let lint_handles: Vec<_> = inputs
        .into_iter()
        .map(|(name, content)| {
            let config = Arc::clone(&config);
            tokio::task::spawn_blocking(move || {
                let errors = lint_content(&content, &config, &name);
                (name, errors)
            })
        })
        .collect();

    for handle in lint_handles {
        let (name, error_result) = handle.await.map_err(|e| {
            MarkdownlintError::AsyncRuntime(format!("Task join error: {}", e))
        })?;
        results.add(name, error_result?);
    }

    Ok(results)
}

/// Load configuration from options
fn load_config(options: &LintOptions) -> Result<Config> {
    if let Some(config) = &options.config {
        Ok(config.clone())
    } else if let Some(config_file) = &options.config_file {
        Config::from_file(config_file)
    } else {
        Ok(Config::default())
    }
}

/// Lint a single piece of content
fn lint_content(content: &str, config: &Config, name: &str) -> Result<Vec<LintError>> {
    use crate::config::RuleConfig;
    use crate::rules;
    use std::collections::HashMap;

    // Parse the markdown
    let tokens = parser::parse(content);

    // Split into lines (preserve line endings)
    let lines: Vec<String> = if content.contains("\r\n") {
        content.split("\r\n").map(|s| format!("{}\r\n", s)).collect()
    } else {
        content.split('\n').map(|s| format!("{}\n", s)).collect()
    };

    // Execute all enabled rules
    let mut all_errors = Vec::new();

    for rule in rules::get_rules() {
        // Check if rule is enabled in config
        let rule_name = rule.names()[0];
        if !config.is_rule_enabled(rule_name) {
            continue;
        }

        // Extract per-rule config options
        let rule_config = match config.get_rule_config(rule_name) {
            Some(RuleConfig::Options(opts)) => opts.clone(),
            _ => HashMap::new(),
        };

        let params = crate::types::RuleParams {
            name,
            version: crate::VERSION,
            lines: &lines,
            front_matter_lines: &[],
            tokens: &tokens,
            config: &rule_config,
        };

        // Run the rule
        let errors = rule.lint(&params);
        all_errors.extend(errors);
    }

    // Sort errors by line number
    all_errors.sort_by_key(|e| e.line_number);

    Ok(all_errors)
}

/// Apply fixes to markdown content
pub fn apply_fixes(content: &str, errors: &[LintError]) -> String {
    use crate::types::FixInfo;

    // Collect only errors that have fix_info
    let mut fixable: Vec<(usize, &FixInfo)> = errors
        .iter()
        .filter_map(|e| {
            e.fix_info.as_ref().map(|fi| {
                let line = fi.line_number.unwrap_or(e.line_number);
                (line, fi)
            })
        })
        .collect();

    if fixable.is_empty() {
        return content.to_string();
    }

    // Split content into lines, preserving line endings
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = if line_ending == "\r\n" {
        content.split("\r\n").map(|s| s.to_string()).collect()
    } else {
        content.split('\n').map(|s| s.to_string()).collect()
    };

    // Remove trailing empty element from split (if content ends with newline)
    if lines.last().is_some_and(|l| l.is_empty()) && content.ends_with(line_ending) {
        lines.pop();
    }

    // Sort fixes: line DESC, then column DESC (apply bottom-up, right-to-left)
    fixable.sort_by(|a, b| {
        b.0.cmp(&a.0).then_with(|| {
            let col_b = b.1.edit_column.unwrap_or(1);
            let col_a = a.1.edit_column.unwrap_or(1);
            col_b.cmp(&col_a)
        })
    });

    // Track which lines have been deleted to avoid double-processing
    let mut deleted_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (line_num, fix) in &fixable {
        let line_idx = line_num.saturating_sub(1);

        // Delete entire line
        if fix.delete_count == Some(-1) {
            if line_idx < lines.len() && !deleted_lines.contains(&line_idx) {
                deleted_lines.insert(line_idx);
            }
            continue;
        }

        if line_idx >= lines.len() || deleted_lines.contains(&line_idx) {
            continue;
        }

        let line = &mut lines[line_idx];
        let col = fix.edit_column.unwrap_or(1);
        let col_idx = col.saturating_sub(1); // Convert 1-based to 0-based

        // Delete characters if specified
        let del = fix.delete_count.unwrap_or(0).max(0) as usize;
        if del > 0 && col_idx < line.len() {
            let end = (col_idx + del).min(line.len());
            line.replace_range(col_idx..end, "");
        }

        // Insert text if specified
        if let Some(ref text) = fix.insert_text {
            let insert_pos = col_idx.min(line.len());
            line.insert_str(insert_pos, text);
        }
    }

    // Remove deleted lines (iterate in reverse to preserve indices)
    let mut deleted_sorted: Vec<usize> = deleted_lines.into_iter().collect();
    deleted_sorted.sort_unstable_by(|a, b| b.cmp(a));
    for idx in deleted_sorted {
        if idx < lines.len() {
            lines.remove(idx);
        }
    }

    // Rejoin with line endings
    let mut result = lines.join(line_ending);
    if content.ends_with(line_ending) {
        result.push_str(line_ending);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FixInfo, Severity};

    #[test]
    fn test_lint_string() {
        let options = LintOptions {
            strings: vec![("test.md".to_string(), "# Hello\n".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let results = lint_sync(&options).unwrap();
        // Verify the file was processed (key exists in results)
        assert!(results.get("test.md").is_some());
    }

    fn make_error(line: usize, fix: FixInfo) -> LintError {
        LintError {
            line_number: line,
            rule_names: vec!["TEST".to_string()],
            rule_description: "test".to_string(),
            fix_info: Some(fix),
            severity: Severity::Error,
            ..Default::default()
        }
    }

    #[test]
    fn test_apply_fixes_trailing_whitespace() {
        // MD009 pattern: delete trailing whitespace
        let content = "hello   \nworld\n";
        let errors = vec![make_error(1, FixInfo {
            line_number: None,
            edit_column: Some(6),
            delete_count: Some(3),
            insert_text: None,
        })];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\nworld\n");
    }

    #[test]
    fn test_apply_fixes_delete_line() {
        // MD012 pattern: delete entire blank line
        let content = "line1\n\n\nline2\n";
        let errors = vec![make_error(2, FixInfo {
            line_number: Some(3),
            edit_column: Some(1),
            delete_count: Some(-1),
            insert_text: None,
        })];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "line1\n\nline2\n");
    }

    #[test]
    fn test_apply_fixes_insert_text() {
        // MD047 pattern: insert newline at end of file
        let content = "hello";
        let errors = vec![make_error(1, FixInfo {
            line_number: Some(1),
            edit_column: Some(6),
            delete_count: None,
            insert_text: Some("\n".to_string()),
        })];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn test_apply_fixes_replace_chars() {
        // MD007 pattern: replace indentation
        let content = "   * item\n";
        let errors = vec![make_error(1, FixInfo {
            line_number: None,
            edit_column: Some(1),
            delete_count: Some(3),
            insert_text: Some("  ".to_string()),
        })];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "  * item\n");
    }

    #[test]
    fn test_apply_fixes_insert_space() {
        // MD018 pattern: insert space after heading marker
        let content = "#heading\n";
        let errors = vec![make_error(1, FixInfo {
            line_number: None,
            edit_column: Some(2),
            delete_count: None,
            insert_text: Some(" ".to_string()),
        })];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "# heading\n");
    }

    #[test]
    fn test_apply_fixes_multiple_lines() {
        let content = "hello   \n#heading\nworld  \n";
        let errors = vec![
            // Trailing whitespace on line 1
            make_error(1, FixInfo {
                line_number: None,
                edit_column: Some(6),
                delete_count: Some(3),
                insert_text: None,
            }),
            // Missing space after # on line 2
            make_error(2, FixInfo {
                line_number: None,
                edit_column: Some(2),
                delete_count: None,
                insert_text: Some(" ".to_string()),
            }),
            // Trailing whitespace on line 3
            make_error(3, FixInfo {
                line_number: None,
                edit_column: Some(6),
                delete_count: Some(2),
                insert_text: None,
            }),
        ];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\n# heading\nworld\n");
    }

    #[test]
    fn test_apply_fixes_no_fixable_errors() {
        let content = "hello\n";
        let errors = vec![LintError {
            line_number: 1,
            rule_names: vec!["TEST".to_string()],
            rule_description: "test".to_string(),
            fix_info: None,
            severity: Severity::Error,
            ..Default::default()
        }];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn test_apply_fixes_empty_errors() {
        let content = "hello\n";
        let result = apply_fixes(content, &[]);
        assert_eq!(result, "hello\n");
    }
}
