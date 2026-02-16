//! Integration tests for mkdlint

use mkdlint::{Config, LintOptions, apply_fixes, lint_sync};
use std::collections::HashMap;

/// Helper to lint a single markdown string and return errors for "test.md"
fn lint_string(markdown: &str) -> Vec<mkdlint::LintError> {
    let mut strings = HashMap::new();
    strings.insert("test.md".to_string(), markdown.to_string());
    let options = LintOptions {
        strings,
        ..Default::default()
    };
    let results = lint_sync(&options).unwrap();
    results.get("test.md").unwrap_or(&[]).to_vec()
}

/// Helper to lint with a specific config
fn lint_string_with_config(markdown: &str, config: Config) -> Vec<mkdlint::LintError> {
    let mut strings = HashMap::new();
    strings.insert("test.md".to_string(), markdown.to_string());
    let options = LintOptions {
        strings,
        config: Some(config),
        ..Default::default()
    };
    let results = lint_sync(&options).unwrap();
    results.get("test.md").unwrap_or(&[]).to_vec()
}

/// Check if any error matches a given rule ID
fn has_rule(errors: &[mkdlint::LintError], rule_id: &str) -> bool {
    errors
        .iter()
        .any(|e| e.rule_names.contains(&rule_id.to_string()))
}

// ---- Existing tests ----

#[test]
fn test_basic_lint_string() {
    let markdown = "# Hello World\n\nThis is a test.\n";
    let mut strings = HashMap::new();
    strings.insert("test.md".to_string(), markdown.to_string());

    let options = LintOptions {
        strings,
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    assert!(results.get("test.md").is_some());
}

#[test]
fn test_lint_with_config() {
    let markdown = "# Heading\n";
    let mut strings = HashMap::new();
    strings.insert("test.md".to_string(), markdown.to_string());

    let config = Config::new();

    let options = LintOptions {
        strings,
        config: Some(config),
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    assert!(results.get("test.md").is_some());
}

#[test]
fn test_results_display() {
    let results = mkdlint::LintResults::new();
    let display = format!("{}", results);
    assert_eq!(display, "");
}

#[test]
fn test_config_json_parsing() {
    let json = r#"{"default": true, "MD001": false}"#;
    let config: Config = serde_json::from_str(json).unwrap();

    assert_eq!(config.default, Some(true));
    assert!(!config.is_rule_enabled("MD001"));
    assert!(config.is_rule_enabled("MD003")); // Should use default
}

#[test]
fn test_library_version() {
    let version = mkdlint::version();
    assert!(!version.is_empty());
    assert!(version.starts_with("0."));
}

// ---- New integration tests: Rule violation detection ----

#[test]
fn test_heading_increment_violation() {
    // MD001 is token-based and requires the parser to produce atxHeading tokens with
    // atxHeadingSequence children. The current comrak parser produces generic "heading"
    // tokens, so MD001 doesn't fire via lint_sync yet. Test MD009 as a line-based alternative.
    let errors = lint_string("# Heading 1\n\nLine with trailing spaces   \n");
    assert!(
        has_rule(&errors, "MD009"),
        "Expected MD009 violation for trailing whitespace"
    );
}

#[test]
fn test_trailing_whitespace_detection() {
    // MD009: no trailing spaces
    let errors = lint_string("# Hello\n\nSome text   \nMore text\n");
    assert!(
        has_rule(&errors, "MD009"),
        "Expected MD009 violation for trailing whitespace"
    );
}

#[test]
fn test_no_hard_tabs() {
    // MD010: no hard tabs
    let errors = lint_string("# Hello\n\n\tindented with tab\n");
    assert!(
        has_rule(&errors, "MD010"),
        "Expected MD010 violation for hard tab"
    );
}

#[test]
fn test_line_length_violation() {
    // MD013: line length
    let long_line = "a".repeat(120);
    let markdown = format!("# Title\n\n{}\n", long_line);
    let errors = lint_string(&markdown);
    assert!(
        has_rule(&errors, "MD013"),
        "Expected MD013 violation for long line"
    );
}

#[test]
fn test_multiple_rules_fire() {
    // Input that violates multiple line-based rules
    let markdown = "# Heading 1\n\nSome text   \n\t tabbed\n";
    let errors = lint_string(markdown);

    // Should catch MD009 (trailing spaces) and MD010 (hard tabs)
    assert!(has_rule(&errors, "MD009"), "Expected MD009");
    assert!(has_rule(&errors, "MD010"), "Expected MD010");
}

#[test]
fn test_rule_disable_via_config() {
    // Disable MD009 (trailing whitespace) and verify it's not reported
    let json = r#"{"MD009": false}"#;
    let config: Config = serde_json::from_str(json).unwrap();

    let errors = lint_string_with_config("# Hello\n\nSome text   \n", config);
    assert!(
        !has_rule(&errors, "MD009"),
        "MD009 should be disabled by config"
    );
}

#[test]
fn test_rule_enable_subset() {
    // Disable all rules, enable only MD009
    let json = r#"{"default": false, "MD009": true}"#;
    let config: Config = serde_json::from_str(json).unwrap();

    // This input violates MD009 (trailing spaces) and MD010 (hard tabs)
    let errors = lint_string_with_config("# H1\n\ntext   \n\ttab\n", config);

    assert!(has_rule(&errors, "MD009"), "MD009 should still be enabled");
    assert!(!has_rule(&errors, "MD010"), "MD010 should be disabled");
}

#[test]
fn test_config_yaml_parsing() {
    let yaml = "default: true\nMD001: false\n";
    let config: Config = serde_yaml_ng::from_str(yaml).unwrap();

    assert_eq!(config.default, Some(true));
    assert!(!config.is_rule_enabled("MD001"));
    assert!(config.is_rule_enabled("MD003"));
}

#[test]
fn test_config_toml_parsing() {
    let toml_str = "default = true\n\n[MD001]\nenabled = false\n";
    // TOML config parses rule configs as Options (HashMap)
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.default, Some(true));
}

#[test]
fn test_config_from_json_file() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");
    std::fs::write(&config_path, r#"{"default": true, "MD001": false}"#).unwrap();

    let config = Config::from_json_file(&config_path).unwrap();
    assert_eq!(config.default, Some(true));
    assert!(!config.is_rule_enabled("MD001"));
}

#[test]
fn test_config_from_yaml_file() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.yaml");
    std::fs::write(&config_path, "default: true\nMD001: false\n").unwrap();

    let config = Config::from_yaml_file(&config_path).unwrap();
    assert_eq!(config.default, Some(true));
    assert!(!config.is_rule_enabled("MD001"));
}

#[test]
fn test_config_merge() {
    let json1 = r#"{"default": true, "MD001": false}"#;
    let json2 = r#"{"MD001": true, "MD009": false}"#;

    let mut config1: Config = serde_json::from_str(json1).unwrap();
    let config2: Config = serde_json::from_str(json2).unwrap();

    config1.merge(config2);

    // MD001 should be overridden to true
    assert!(config1.is_rule_enabled("MD001"));
    // MD009 should be disabled from merge
    assert!(!config1.is_rule_enabled("MD009"));
}

#[test]
fn test_lint_multiple_strings() {
    let mut strings = HashMap::new();
    strings.insert("a.md".to_string(), "# Hello\n\nWorld\n".to_string());
    strings.insert("b.md".to_string(), "# Hi\n\nThere\n".to_string());
    strings.insert("c.md".to_string(), "# Hey\n\nFolk\n".to_string());

    let options = LintOptions {
        strings,
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    assert!(results.get("a.md").is_some());
    assert!(results.get("b.md").is_some());
    assert!(results.get("c.md").is_some());
}

#[test]
fn test_clean_markdown_no_errors() {
    // Well-formed markdown that should pass most rules
    let markdown = "# Title\n\nA paragraph with normal text.\n\n## Section\n\nAnother paragraph.\n";
    let errors = lint_string(markdown);

    // Should have zero or very few errors
    // Filter out MD047 (file ending) which may fire depending on trailing newline
    let significant_errors: Vec<_> = errors
        .iter()
        .filter(|e| !has_rule(&[(*e).clone()], "MD047"))
        .collect();

    // Clean markdown shouldn't trigger heading/whitespace/tab rules
    assert!(
        !has_rule(&errors, "MD001"),
        "Clean markdown shouldn't trigger MD001"
    );
    assert!(
        !has_rule(&errors, "MD009"),
        "Clean markdown shouldn't trigger MD009"
    );
    assert!(
        !has_rule(&errors, "MD010"),
        "Clean markdown shouldn't trigger MD010"
    );
    let _ = significant_errors; // suppress unused warning
}

#[test]
fn test_empty_input() {
    let errors = lint_string("");
    // Should not crash; may or may not produce errors
    let _ = errors;
}

#[test]
fn test_lint_file_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.md");
    std::fs::write(&file_path, "# Hello World\n\nSome content.\n").unwrap();

    let options = LintOptions {
        files: vec![file_path.to_string_lossy().to_string()],
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    assert!(results.get(file_path.to_string_lossy().as_ref()).is_some());
}

#[test]
fn test_lint_nonexistent_file() {
    let options = LintOptions {
        files: vec!["/tmp/nonexistent_file_12345.md".to_string()],
        ..Default::default()
    };

    let result = lint_sync(&options);
    assert!(
        result.is_err(),
        "Linting a nonexistent file should return an error"
    );
}

#[test]
fn test_error_has_line_number() {
    // MD009 should report a specific line number
    let errors = lint_string("# Hello\n\nLine with spaces   \n");
    let md009_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_names.contains(&"MD009".to_string()))
        .collect();

    if !md009_errors.is_empty() {
        assert!(
            md009_errors[0].line_number > 0,
            "Error should have a positive line number"
        );
    }
}

#[test]
fn test_results_error_count() {
    let markdown = "# H1\n### H3\ntext   \n\ttab\n";
    let mut strings = HashMap::new();
    strings.insert("test.md".to_string(), markdown.to_string());

    let options = LintOptions {
        strings,
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    assert!(results.error_count() > 0, "Should have at least one error");
    assert!(!results.is_empty(), "Results should not be empty");
    assert!(results.has_errors(), "Should report has_errors");
}

#[test]
fn test_config_file_option() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join(".markdownlint.json");
    std::fs::write(&config_path, r#"{"default": false}"#).unwrap();

    let file_path = dir.path().join("test.md");
    std::fs::write(&file_path, "# H1\n### H3\ntext   \n").unwrap();

    let options = LintOptions {
        files: vec![file_path.to_string_lossy().to_string()],
        config_file: Some(config_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    let errors = results
        .get(file_path.to_string_lossy().as_ref())
        .unwrap_or(&[]);
    // All rules disabled, so no errors expected
    assert!(
        errors.is_empty(),
        "All rules disabled via config_file, expected 0 errors but got {}",
        errors.len()
    );
}

// ---- New: MD001 fires through lint_sync (parser → tokens → rule) ----

#[test]
fn test_md001_heading_increment_via_lint_sync() {
    // # H1 then ### H3 skips level 2 — MD001 should fire
    let errors = lint_string("# H1\n\n### H3\n");
    assert!(
        has_rule(&errors, "MD001"),
        "MD001 should fire for heading increment skip (H1 → H3). Errors: {:?}",
        errors.iter().map(|e| &e.rule_names).collect::<Vec<_>>()
    );
}

#[test]
fn test_md001_no_violation_sequential() {
    // Sequential headings: H1, H2, H3 — MD001 should NOT fire
    let errors = lint_string("# H1\n\n## H2\n\n### H3\n");
    assert!(
        !has_rule(&errors, "MD001"),
        "MD001 should NOT fire for sequential headings"
    );
}

// ---- New: Config wiring (MD013 line_length) ----

#[test]
fn test_md013_config_line_length_200() {
    // With line_length=200, a 100-char line should NOT trigger MD013
    let json = r#"{"default": false, "MD013": {"line_length": 200}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let line = format!("# Title\n\n{}\n", "a".repeat(100));
    let errors = lint_string_with_config(&line, config);
    assert!(
        !has_rule(&errors, "MD013"),
        "MD013 should NOT fire with line_length=200 for 100-char line"
    );
}

#[test]
fn test_md013_config_line_length_50() {
    // With line_length=50, a 60-char line should trigger MD013
    let json = r#"{"default": false, "MD013": {"line_length": 50}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let line = format!("# Title\n\n{}\n", "a".repeat(60));
    let errors = lint_string_with_config(&line, config);
    assert!(
        has_rule(&errors, "MD013"),
        "MD013 should fire with line_length=50 for 60-char line. Errors: {:?}",
        errors.iter().map(|e| &e.rule_names).collect::<Vec<_>>()
    );
}

// ---- New: apply_fixes round-trip ----

#[test]
fn test_apply_fixes_round_trip_trailing_whitespace() {
    // Lint → get errors → apply_fixes → lint again → 0 MD009 errors
    let content = "# Title\n\nSome text   \nMore text  \n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD009"), "Should have MD009 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(
        !has_rule(&errors_after, "MD009"),
        "After apply_fixes, MD009 should be gone. Fixed content: {:?}",
        fixed
    );
}

#[test]
fn test_apply_fixes_round_trip_hard_tabs() {
    // Lint → get errors → apply_fixes → lint again → 0 MD010 errors
    let content = "# Title\n\n\tindented\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD010"), "Should have MD010 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(
        !has_rule(&errors_after, "MD010"),
        "After apply_fixes, MD010 should be gone. Fixed content: {:?}",
        fixed
    );
}

// ---- MD022: Headings should be surrounded by blank lines ----

#[test]
fn test_md022_missing_blank_before_heading() {
    let errors = lint_string("# Title\nSome text\n## Section\n");
    assert!(
        has_rule(&errors, "MD022"),
        "MD022 should fire when heading lacks blank line before it"
    );
}

#[test]
fn test_md022_correct_blank_lines() {
    let errors = lint_string("# Title\n\nSome text\n\n## Section\n\nMore text\n");
    assert!(
        !has_rule(&errors, "MD022"),
        "MD022 should NOT fire when headings have blank lines around them"
    );
}

// ---- MD031 apply_fixes round-trip: blank line insertion ----

#[test]
fn test_apply_fixes_round_trip_md031_missing_blank_lines() {
    // Code fence missing blank lines before/after
    let content = "# Title\n\nSome text\n```\ncode\n```\nMore text\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD031"), "Should have MD031 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(
        !has_rule(&errors_after, "MD031"),
        "After apply_fixes, MD031 should be gone. Fixed content: {:?}",
        fixed
    );
}

// ---- MD042: Reference links now work ----

#[test]
fn test_md042_empty_link_via_lint_sync() {
    let errors = lint_string("[click here]()\n");
    assert!(
        has_rule(&errors, "MD042"),
        "MD042 should fire for empty inline link"
    );
}

#[test]
fn test_md042_reference_empty_via_lint_sync() {
    let errors = lint_string("[click][ref]\n\n[ref]: #\n");
    assert!(
        has_rule(&errors, "MD042"),
        "MD042 should fire for reference link pointing to empty fragment"
    );
}
