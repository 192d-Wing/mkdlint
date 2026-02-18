//! Core linting functionality

use crate::config::Config;
use crate::parser;
use crate::types::{
    BoxedRule, LintError, LintOptions, LintResults, MarkdownlintError, ParserType, Result,
};
use rayon::prelude::*;

/// Default maximum number of fix passes for convergence
pub const DEFAULT_FIX_PASSES: usize = 10;

/// Pre-computed rule state for a given configuration.
///
/// Built once per lint invocation and shared across all files,
/// avoiding redundant HashMap lookups and Vec allocations per file.
struct PreparedRules<'a> {
    enabled: Vec<&'a BoxedRule>,
    needs_parser: bool,
    front_matter_pattern: Option<String>,
}

/// Build the enabled-rules list and parser flag from the config.
///
/// Returns `'static` because rule references come from the global registry.
fn prepare_rules(config: &Config, front_matter_pattern: Option<String>) -> PreparedRules<'static> {
    use crate::rules;

    let enabled: Vec<&BoxedRule> = rules::get_rules()
        .iter()
        .filter(|rule| {
            let explicitly_configured = config.get_rule_config(rule.names()[0]).is_some();
            if explicitly_configured {
                config.is_rule_enabled(rule.names()[0])
            } else {
                // No explicit config entry: use the rule's own default,
                // but still respect the global `default` override if set.
                config
                    .default
                    .unwrap_or_else(|| rule.is_enabled_by_default())
            }
        })
        .collect();

    let needs_parser = enabled
        .iter()
        .any(|rule| rule.parser_type() == ParserType::Micromark);

    PreparedRules {
        enabled,
        needs_parser,
        front_matter_pattern,
    }
}

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

    // Precompute enabled rules once (avoids per-file HashMap lookups)
    let prepared = prepare_rules(&config, options.front_matter.clone());

    // Lint all inputs in parallel
    let file_results: Vec<(
        String,
        std::result::Result<Vec<LintError>, MarkdownlintError>,
    )> = inputs
        .par_iter()
        .map(|(name, content)| {
            let errors = lint_content(content, &config, name, &prepared);
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
        let (path, content_result) = handle
            .await
            .map_err(|e| MarkdownlintError::AsyncRuntime(format!("Task join error: {}", e)))?;
        inputs.push((path, content_result?));
    }

    // Add string inputs
    for (name, content) in &options.strings {
        inputs.push((name.clone(), content.clone()));
    }

    // Precompute enabled rules once
    let prepared = Arc::new(prepare_rules(&config, options.front_matter.clone()));

    // Lint all inputs concurrently using spawn_blocking (CPU-bound)
    let lint_handles: Vec<_> = inputs
        .into_iter()
        .map(|(name, content)| {
            let config = Arc::clone(&config);
            let prepared = Arc::clone(&prepared);
            tokio::task::spawn_blocking(move || {
                let errors = lint_content(&content, &config, &name, &prepared);
                (name, errors)
            })
        })
        .collect();

    for handle in lint_handles {
        let (name, error_result) = handle
            .await
            .map_err(|e| MarkdownlintError::AsyncRuntime(format!("Task join error: {}", e)))?;
        results.add(name, error_result?);
    }

    Ok(results)
}

/// Load configuration from options
fn load_config(options: &LintOptions) -> Result<Config> {
    let config = if let Some(config) = &options.config {
        config.clone()
    } else if let Some(config_file) = &options.config_file {
        Config::from_file(config_file)?
    } else {
        // Auto-discover from first file's parent directory or CWD
        let start = options
            .files
            .first()
            .and_then(|f| std::path::Path::new(f).parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        Config::discover(&start).unwrap_or_default()
    };

    // Resolve extends chain
    config.resolve_extends()
}

/// Extract front matter line count from document.
///
/// Supports custom regex pattern. When pattern is None, no front matter is extracted
/// (for backwards compatibility - user must opt-in via --front-matter flag).
/// Returns the number of lines in the front matter block (including delimiters),
/// or 0 if no front matter is detected.
fn extract_front_matter_line_count(lines: &[&str], pattern: Option<&str>) -> usize {
    if lines.is_empty() {
        return 0;
    }

    let first_line = lines[0].trim_end_matches(['\n', '\r']);

    // Only extract front matter when explicitly requested via pattern
    let pattern_str = match pattern {
        Some(p) => p,
        None => return 0, // No pattern = no front matter extraction (opt-in only)
    };

    let Ok(regex) = regex::Regex::new(pattern_str) else {
        return 0;
    };
    if !regex.is_match(first_line) {
        return 0;
    }
    // Scan for closing delimiter (second pattern match)
    for i in 1..lines.len() {
        let line = lines[i].trim_end_matches(['\n', '\r']);
        if regex.is_match(line) {
            return i + 1;
        }
    }
    0 // No closing = no front matter
}

/// Lint a single piece of content using pre-computed rule state.
fn lint_content(
    content: &str,
    config: &Config,
    name: &str,
    prepared: &PreparedRules<'_>,
) -> Result<Vec<LintError>> {
    use crate::config::RuleConfig;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;

    static EMPTY_CONFIG: Lazy<HashMap<String, serde_json::Value>> = Lazy::new(HashMap::new);

    // Split into lines (zero-copy, preserving line endings)
    let lines: Vec<&str> = content.split_inclusive('\n').collect();

    // Extract front matter if present
    let fm_count =
        extract_front_matter_line_count(&lines, prepared.front_matter_pattern.as_deref());
    let front_matter_lines: &[&str] = &lines[..fm_count];

    // Parse inline configuration directives (<!-- markdownlint-disable/enable -->)
    let inline_config = InlineConfig::parse(&lines);

    let mut all_errors = Vec::new();

    // Only parse if at least one enabled rule needs tokens
    let tokens = if prepared.needs_parser {
        parser::parse(content)
    } else {
        vec![]
    };

    for rule in &prepared.enabled {
        let rule_name = rule.names()[0];

        // Extract per-rule config options (avoid clone when no config)
        let rule_config = match config.get_rule_config(rule_name) {
            Some(RuleConfig::Options(opts)) => opts,
            _ => &EMPTY_CONFIG,
        };

        let params = crate::types::RuleParams {
            name,
            version: crate::VERSION,
            lines: &lines,
            front_matter_lines,
            tokens: &tokens,
            config: rule_config,
        };

        // Run the rule
        let mut errors = rule.lint(&params);

        // Apply per-rule severity override from config (if set)
        if let Some(severity) = config.get_rule_severity(rule_name) {
            for error in &mut errors {
                error.severity = severity;
            }
        }

        all_errors.extend(errors);
    }

    // Filter out errors suppressed by inline configuration
    if inline_config.has_directives {
        all_errors.retain(|error| !inline_config.is_disabled(error.line_number, error.rule_names));
    }

    // Sort errors by line number
    all_errors.sort_by_key(|e| e.line_number);

    Ok(all_errors)
}

// ---------------------------------------------------------------------------
// Inline configuration directives
// ---------------------------------------------------------------------------

/// Parsed inline configuration state.
///
/// Uses a snapshot-based approach: instead of cloning rule ID strings into
/// per-line HashSets (O(lines × rules) allocations), we store directive
/// events and evaluate `is_disabled()` lazily by scanning events.
///
/// Supports the following HTML comment directives:
/// - `<!-- markdownlint-disable MD001 MD002 -->` — disable specific rules
/// - `<!-- markdownlint-disable -->` — disable all rules
/// - `<!-- markdownlint-enable MD001 -->` — re-enable specific rules
/// - `<!-- markdownlint-enable -->` — re-enable all rules
/// - `<!-- markdownlint-disable-next-line MD001 -->` — disable for next line only
/// - `<!-- markdownlint-disable-file MD001 -->` — disable for entire file
/// - `<!-- markdownlint-enable-file MD001 -->` — re-enable for rest of file
struct InlineConfig {
    /// Whether any directives were found (fast path for skipping filter).
    has_directives: bool,
    /// Sorted directive events (line_number, event). Always sorted by line_number.
    events: Vec<(usize, DirectiveEvent)>,
}

use std::collections::HashSet;

/// A single inline directive event, stored once during parse.
enum DirectiveEvent {
    Disable(Vec<String>),
    Enable(Vec<String>),
    DisableNextLine(Vec<String>),
    DisableFile(Vec<String>),
    EnableFile(Vec<String>),
}

impl InlineConfig {
    /// Parse inline directives from document lines.
    fn parse(lines: &[&str]) -> Self {
        let mut has_directives = false;
        let mut events = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if let Some(directive) = Self::parse_directive(trimmed) {
                has_directives = true;
                let event = match directive {
                    Directive::Disable(rules) => DirectiveEvent::Disable(rules),
                    Directive::Enable(rules) => DirectiveEvent::Enable(rules),
                    Directive::DisableNextLine(rules) => DirectiveEvent::DisableNextLine(rules),
                    Directive::DisableFile(rules) => DirectiveEvent::DisableFile(rules),
                    Directive::EnableFile(rules) => DirectiveEvent::EnableFile(rules),
                };
                events.push((line_number, event));
            }
        }

        InlineConfig {
            has_directives,
            events,
        }
    }

    /// Check if a rule is disabled at a given line.
    ///
    /// Replays directive events up to `line_number` to compute the disabled
    /// state. This avoids the O(lines × rules) String cloning of the
    /// previous per-line HashSet approach.
    fn is_disabled(&self, line_number: usize, rule_names: &[&str]) -> bool {
        let mut active_disabled: HashSet<&str> = HashSet::new();
        let mut file_disabled: HashSet<&str> = HashSet::new();
        // Track the line number of the last disable-next-line directive
        let mut disable_next_line: Option<(usize, &[String])> = None;

        for (event_line, event) in &self.events {
            if *event_line >= line_number {
                break;
            }
            match event {
                DirectiveEvent::Disable(rules) => {
                    if rules.is_empty() {
                        active_disabled.insert("");
                    } else {
                        for r in rules {
                            active_disabled.insert(r);
                        }
                    }
                }
                DirectiveEvent::Enable(rules) => {
                    if rules.is_empty() {
                        active_disabled.clear();
                    } else {
                        for r in rules {
                            active_disabled.remove(r.as_str());
                        }
                    }
                }
                DirectiveEvent::DisableNextLine(rules) => {
                    disable_next_line = Some((*event_line, rules));
                }
                DirectiveEvent::DisableFile(rules) => {
                    if rules.is_empty() {
                        file_disabled.insert("");
                    } else {
                        for r in rules {
                            file_disabled.insert(r);
                        }
                    }
                }
                DirectiveEvent::EnableFile(rules) => {
                    if rules.is_empty() {
                        file_disabled.clear();
                    } else {
                        for r in rules {
                            file_disabled.remove(r.as_str());
                        }
                    }
                }
            }
        }

        // Check file-level disables
        if file_disabled.contains("") {
            return true;
        }
        for name in rule_names {
            if file_disabled.contains(name) {
                return true;
            }
        }

        // Check sticky disable/enable
        if active_disabled.contains("") {
            return true;
        }
        for name in rule_names {
            if active_disabled.contains(name) {
                return true;
            }
        }

        // Check disable-next-line: applies to the first non-directive line
        // after the directive. We need to find if line_number is the target.
        if let Some((dnl_line, rules)) = disable_next_line {
            // The disable-next-line applies to the next non-directive line
            // after dnl_line. Find it by checking if any events exist between
            // dnl_line and line_number.
            let next_non_directive = self.find_next_non_directive_line(dnl_line);
            if next_non_directive == Some(line_number) {
                if rules.is_empty() {
                    return true;
                }
                for name in rule_names {
                    if rules.iter().any(|r| r == name) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Find the first non-directive line after `after_line`.
    fn find_next_non_directive_line(&self, after_line: usize) -> Option<usize> {
        // Collect all directive line numbers
        let directive_lines: HashSet<usize> = self.events.iter().map(|(l, _)| *l).collect();
        let mut line = after_line + 1;
        // Skip consecutive directive lines
        while directive_lines.contains(&line) {
            line += 1;
        }
        Some(line)
    }

    /// Parse a single directive from a trimmed line.
    fn parse_directive(line: &str) -> Option<Directive> {
        // Must be an HTML comment: <!-- markdownlint-xxx ... -->
        let inner = line.strip_prefix("<!--")?.strip_suffix("-->")?.trim();

        if let Some(rest) = inner.strip_prefix("markdownlint-disable-next-line") {
            let rules = Self::parse_rule_list(rest);
            Some(Directive::DisableNextLine(rules))
        } else if let Some(rest) = inner.strip_prefix("markdownlint-disable-file") {
            let rules = Self::parse_rule_list(rest);
            Some(Directive::DisableFile(rules))
        } else if let Some(rest) = inner.strip_prefix("markdownlint-enable-file") {
            let rules = Self::parse_rule_list(rest);
            Some(Directive::EnableFile(rules))
        } else if let Some(rest) = inner.strip_prefix("markdownlint-disable") {
            let rules = Self::parse_rule_list(rest);
            Some(Directive::Disable(rules))
        } else if let Some(rest) = inner.strip_prefix("markdownlint-enable") {
            let rules = Self::parse_rule_list(rest);
            Some(Directive::Enable(rules))
        } else {
            None
        }
    }

    /// Parse a space-separated list of rule IDs from directive content.
    fn parse_rule_list(s: &str) -> Vec<String> {
        s.split_whitespace().map(|r| r.to_uppercase()).collect()
    }
}

enum Directive {
    Disable(Vec<String>),
    Enable(Vec<String>),
    DisableNextLine(Vec<String>),
    DisableFile(Vec<String>),
    EnableFile(Vec<String>),
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
    let line_ending = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
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

    // Track which lines have been deleted or structurally modified
    let mut deleted_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();
    // Lines where a newline was inserted — subsequent fixes would operate on
    // shifted content, so we skip them (they'll be caught on the next lint pass).
    let mut restructured_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (line_num, fix) in &fixable {
        let line_idx = line_num.saturating_sub(1);

        // Delete entire line
        if fix.delete_count == Some(-1) {
            if line_idx < lines.len() && !deleted_lines.contains(&line_idx) {
                deleted_lines.insert(line_idx);
            }
            continue;
        }

        if line_idx >= lines.len()
            || deleted_lines.contains(&line_idx)
            || restructured_lines.contains(&line_idx)
        {
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
            // Normalize newlines in inserted text to match the document's style
            if line_ending == "\r\n" && text.contains('\n') && !text.contains("\r\n") {
                let normalized = text.replace('\n', "\r\n");
                line.insert_str(insert_pos, &normalized);
            } else {
                line.insert_str(insert_pos, text);
            }

            // If inserted text contains a newline, mark the line as restructured
            // so subsequent fixes don't operate on shifted content
            if text.contains('\n') {
                restructured_lines.insert(line_idx);
            }
        }
    }

    // Remove deleted lines in a single pass
    if !deleted_lines.is_empty() {
        let mut idx = 0;
        lines.retain(|_| {
            let keep = !deleted_lines.contains(&idx);
            idx += 1;
            keep
        });
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
            rule_names: &["TEST"],
            rule_description: "test",
            fix_info: Some(fix),
            severity: Severity::Error,
            fix_only: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_apply_fixes_trailing_whitespace() {
        // MD009 pattern: delete trailing whitespace
        let content = "hello   \nworld\n";
        let errors = vec![make_error(
            1,
            FixInfo {
                line_number: None,
                edit_column: Some(6),
                delete_count: Some(3),
                insert_text: None,
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\nworld\n");
    }

    #[test]
    fn test_apply_fixes_delete_line() {
        // MD012 pattern: delete entire blank line
        let content = "line1\n\n\nline2\n";
        let errors = vec![make_error(
            2,
            FixInfo {
                line_number: Some(3),
                edit_column: Some(1),
                delete_count: Some(-1),
                insert_text: None,
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "line1\n\nline2\n");
    }

    #[test]
    fn test_apply_fixes_insert_text() {
        // MD047 pattern: insert newline at end of file
        let content = "hello";
        let errors = vec![make_error(
            1,
            FixInfo {
                line_number: Some(1),
                edit_column: Some(6),
                delete_count: None,
                insert_text: Some("\n".to_string()),
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn test_apply_fixes_replace_chars() {
        // MD007 pattern: replace indentation
        let content = "   * item\n";
        let errors = vec![make_error(
            1,
            FixInfo {
                line_number: None,
                edit_column: Some(1),
                delete_count: Some(3),
                insert_text: Some("  ".to_string()),
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "  * item\n");
    }

    #[test]
    fn test_apply_fixes_insert_space() {
        // MD018 pattern: insert space after heading marker
        let content = "#heading\n";
        let errors = vec![make_error(
            1,
            FixInfo {
                line_number: None,
                edit_column: Some(2),
                delete_count: None,
                insert_text: Some(" ".to_string()),
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "# heading\n");
    }

    #[test]
    fn test_apply_fixes_multiple_lines() {
        let content = "hello   \n#heading\nworld  \n";
        let errors = vec![
            // Trailing whitespace on line 1
            make_error(
                1,
                FixInfo {
                    line_number: None,
                    edit_column: Some(6),
                    delete_count: Some(3),
                    insert_text: None,
                },
            ),
            // Missing space after # on line 2
            make_error(
                2,
                FixInfo {
                    line_number: None,
                    edit_column: Some(2),
                    delete_count: None,
                    insert_text: Some(" ".to_string()),
                },
            ),
            // Trailing whitespace on line 3
            make_error(
                3,
                FixInfo {
                    line_number: None,
                    edit_column: Some(6),
                    delete_count: Some(2),
                    insert_text: None,
                },
            ),
        ];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\n# heading\nworld\n");
    }

    #[test]
    fn test_apply_fixes_no_fixable_errors() {
        let content = "hello\n";
        let errors = vec![LintError {
            line_number: 1,
            rule_names: &["TEST"],
            rule_description: "test",
            fix_info: None,
            severity: Severity::Error,
            fix_only: false,
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

    #[test]
    fn test_apply_fixes_crlf_trailing_whitespace() {
        // MD009 pattern with CRLF line endings
        let content = "hello   \r\nworld\r\n";
        let errors = vec![make_error(
            1,
            FixInfo {
                line_number: None,
                edit_column: Some(6),
                delete_count: Some(3),
                insert_text: None,
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "hello\r\nworld\r\n");
    }

    #[test]
    fn test_apply_fixes_crlf_insert_newline() {
        // MD047/MD022 pattern: inserting "\n" in CRLF document should become "\r\n"
        let content = "# Title\r\nhello";
        let errors = vec![make_error(
            2,
            FixInfo {
                line_number: Some(2),
                edit_column: Some(6),
                delete_count: None,
                insert_text: Some("\n".to_string()),
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "# Title\r\nhello\r\n");
    }

    #[test]
    fn test_apply_fixes_crlf_insert_multiline() {
        // MD041 pattern: inserting "# Title\n\n" in CRLF document
        let content = "Some text\r\n";
        let errors = vec![make_error(
            1,
            FixInfo {
                line_number: Some(1),
                edit_column: Some(1),
                delete_count: None,
                insert_text: Some("# Title\n\n".to_string()),
            },
        )];
        let result = apply_fixes(content, &errors);
        assert_eq!(result, "# Title\r\n\r\nSome text\r\n");
    }

    #[test]
    fn test_extract_front_matter_no_pattern() {
        let lines = vec!["---", "title: Test", "---", "# Content"];
        assert_eq!(extract_front_matter_line_count(&lines, None), 0);
    }

    #[test]
    fn test_extract_front_matter_yaml() {
        let lines = vec!["---\n", "title: Test\n", "---\n", "# Content\n"];
        assert_eq!(extract_front_matter_line_count(&lines, Some("^---$")), 3);
    }

    #[test]
    fn test_extract_front_matter_toml() {
        let lines = vec!["+++\n", "title = \"Test\"\n", "+++\n", "# Content\n"];
        assert_eq!(
            extract_front_matter_line_count(&lines, Some("^\\+\\+\\+$")),
            3
        );
    }

    #[test]
    fn test_extract_front_matter_unclosed() {
        let lines = vec!["---\n", "title: Test\n", "# Content\n"];
        assert_eq!(extract_front_matter_line_count(&lines, Some("^---$")), 0);
    }

    #[test]
    fn test_extract_front_matter_empty_doc() {
        let lines: Vec<&str> = vec![];
        assert_eq!(extract_front_matter_line_count(&lines, Some("^---$")), 0);
    }

    #[test]
    fn test_extract_front_matter_invalid_regex() {
        let lines = vec!["---\n", "title: Test\n", "---\n"];
        assert_eq!(extract_front_matter_line_count(&lines, Some("[")), 0);
    }
}
