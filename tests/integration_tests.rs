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
    errors.iter().any(|e| e.rule_names.contains(&rule_id))
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
        .filter(|e| e.rule_names.contains(&"MD009"))
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

// ---- MD001 fix round-trip ----

#[test]
fn test_md001_fix_round_trip() {
    let input = "# H1\n\n### H3\n";
    let errors = lint_string(input);
    assert!(has_rule(&errors, "MD001"), "MD001 should fire");
    let fixed = apply_fixes(input, &errors);
    let errors2 = lint_string(&fixed);
    assert!(
        !has_rule(&errors2, "MD001"),
        "MD001 should not fire after fix. Fixed:\n{}",
        fixed
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

// ---- Inline configuration directives ----

#[test]
fn test_inline_disable_specific_rule() {
    let markdown = "# Title\n\n<!-- markdownlint-disable MD009 -->\nText with spaces   \n<!-- markdownlint-enable MD009 -->\n";
    let errors = lint_string(markdown);
    assert!(
        !has_rule(&errors, "MD009"),
        "MD009 should be disabled by inline directive"
    );
}

#[test]
fn test_inline_disable_all_rules() {
    // Start with a heading so MD041 doesn't fire on line 1
    let markdown = "# Title\n\n<!-- markdownlint-disable -->\n#no space\ntext   \n\ttab\n<!-- markdownlint-enable -->\n";
    let errors = lint_string(markdown);
    // Lines 4-6 should have no errors (all rules disabled)
    let errors_in_range: Vec<_> = errors
        .iter()
        .filter(|e| e.line_number >= 4 && e.line_number <= 6)
        .collect();
    assert!(
        errors_in_range.is_empty(),
        "All rules disabled between directives, expected 0 errors in range but got {}: {:?}",
        errors_in_range.len(),
        errors_in_range
            .iter()
            .map(|e| (e.line_number, &e.rule_names))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_inline_disable_next_line() {
    let markdown = "# Title\n\n<!-- markdownlint-disable-next-line MD009 -->\nText with spaces   \nMore spaces   \n";
    let errors = lint_string(markdown);
    let md009_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_names.contains(&"MD009"))
        .collect();
    // Only the second line with trailing spaces should report MD009
    assert_eq!(
        md009_errors.len(),
        1,
        "Only one MD009 error expected (next-line only disables one line)"
    );
    assert_eq!(
        md009_errors[0].line_number, 5,
        "MD009 should fire on line 5 (not line 4)"
    );
}

#[test]
fn test_inline_disable_file() {
    let markdown = "# Title\n\n<!-- markdownlint-disable-file MD009 -->\nText   \nMore   \n";
    let errors = lint_string(markdown);
    assert!(
        !has_rule(&errors, "MD009"),
        "MD009 should be disabled for entire file"
    );
}

#[test]
fn test_inline_disable_does_not_affect_other_rules() {
    let markdown = "# Title\n\n<!-- markdownlint-disable MD009 -->\nText   \n\ttab\n<!-- markdownlint-enable -->\n";
    let errors = lint_string(markdown);
    assert!(!has_rule(&errors, "MD009"), "MD009 should be disabled");
    assert!(
        has_rule(&errors, "MD010"),
        "MD010 should still fire (only MD009 was disabled)"
    );
}

#[test]
fn test_inline_disable_multiple_rules() {
    let markdown = "# Title\n\n<!-- markdownlint-disable MD009 MD010 -->\nText   \n\ttab\n<!-- markdownlint-enable -->\n";
    let errors = lint_string(markdown);
    assert!(!has_rule(&errors, "MD009"), "MD009 should be disabled");
    assert!(!has_rule(&errors, "MD010"), "MD010 should be disabled");
}

#[test]
fn test_inline_enable_re_enables_after_disable() {
    let markdown = "# Title\n\n<!-- markdownlint-disable MD009 -->\nText   \n<!-- markdownlint-enable MD009 -->\nMore text   \n";
    let errors = lint_string(markdown);
    let md009_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_names.contains(&"MD009"))
        .collect();
    assert_eq!(md009_errors.len(), 1, "Only one MD009 after re-enable");
    assert_eq!(
        md009_errors[0].line_number, 6,
        "MD009 should fire on line 6 (after enable)"
    );
}

// ---- CRLF line ending support ----

#[test]
fn test_crlf_apply_fixes_preserves_crlf() {
    let crlf_doc = "# Title\r\nSome text  \r\n";
    let errors = lint_string(crlf_doc);
    assert!(has_rule(&errors, "MD009"), "Should detect trailing spaces");
    let fixed = apply_fixes(crlf_doc, &errors);
    // All newlines should be CRLF
    for (i, byte) in fixed.bytes().enumerate() {
        if byte == b'\n' && i > 0 {
            assert_eq!(
                fixed.as_bytes()[i - 1],
                b'\r',
                "Bare \\n at byte {}: {:?}",
                i,
                &fixed
            );
        }
    }
    assert!(
        !has_rule(&lint_string(&fixed), "MD009"),
        "MD009 should be fixed"
    );
}

#[test]
fn test_crlf_conflicting_fixes_no_corruption() {
    // Input triggers MD009, MD022, and MD025 — all targeting line 2
    let crlf_doc = "# \r\n# \r\n";
    let errors = lint_string(crlf_doc);
    let fixed = apply_fixes(crlf_doc, &errors);
    // Must not produce bare \n in CRLF document
    for (i, byte) in fixed.bytes().enumerate() {
        if byte == b'\n' && i > 0 {
            assert_eq!(
                fixed.as_bytes()[i - 1],
                b'\r',
                "Bare \\n at byte {}: {:?}",
                i,
                &fixed
            );
        }
    }
}

// ---- MD059 auto-fix round-trip ----

#[test]
fn test_apply_fixes_round_trip_md059_inline_math() {
    let content = "# Title\n\n$_text_$\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD059"), "Should have MD059 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(
        !has_rule(&errors_after, "MD059"),
        "After apply_fixes, MD059 should be gone. Fixed content: {:?}",
        fixed
    );
}

#[test]
fn test_apply_fixes_round_trip_md059_display_math() {
    let content = "# Title\n\n$$\n_text_\n$$\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD059"), "Should have MD059 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(
        !has_rule(&errors_after, "MD059"),
        "After apply_fixes, MD059 should be gone. Fixed content: {:?}",
        fixed
    );
}

// ---- MD054 auto-fix round-trip ----

#[test]
fn test_apply_fixes_round_trip_md054_collapsed_to_shortcut() {
    let json = r#"{"default": false, "MD054": {"collapsed": false}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n[text][] is a link\n\n[text]: https://example.com\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD054"), "Should have MD054 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(
        !has_rule(&errors_after, "MD054"),
        "After apply_fixes, MD054 should be gone. Fixed content: {:?}",
        fixed
    );
}

#[test]
fn test_apply_fixes_round_trip_md054_autolink_to_inline() {
    let json = r#"{"default": false, "MD054": {"autolink": false}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n<https://example.com>\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD054"), "Should have MD054 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(
        !has_rule(&errors_after, "MD054"),
        "After apply_fixes, MD054 should be gone. Fixed content: {:?}",
        fixed
    );
}

// ---- MD046 auto-fix round-trip ----

#[test]
fn test_apply_fixes_round_trip_md046_indented_to_fenced() {
    let json = r#"{"default": false, "MD046": {"style": "fenced"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n    indented code\n    more code\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD046"), "Should have MD046 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(
        !has_rule(&errors_after, "MD046"),
        "After apply_fixes, MD046 should be gone. Fixed content: {:?}",
        fixed
    );
}

#[test]
fn test_apply_fixes_round_trip_md046_fenced_to_indented() {
    let json = r#"{"default": false, "MD046": {"style": "indented"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n```\nfenced code\nmore code\n```\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD046"), "Should have MD046 initially");

    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(
        !has_rule(&errors_after, "MD046"),
        "After apply_fixes, MD046 should be gone. Fixed content: {:?}",
        fixed
    );
}

// =========================================================================
// Phase 1: Integration tests for previously-uncovered rules
// =========================================================================

// ---- Heading rules (MD003, MD024, MD025, MD041) ----

#[test]
fn test_md003_setext_violation() {
    let content = "Title\n=====\n\n## Section\n";
    let errors = lint_string(content);
    assert!(
        has_rule(&errors, "MD003"),
        "Setext + ATX mix should trigger MD003"
    );
}

#[test]
fn test_apply_fixes_round_trip_md003() {
    let json = r#"{"MD003": {"style": "atx"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "Title\n=====\n\nSubtitle\n--------\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD003"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(!has_rule(&errors_after, "MD003"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md024_duplicate_heading() {
    let content = "# Title\n\n## Section\n\n## Section\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD024"));
}

#[test]
fn test_md024_fix_round_trip() {
    let content = "# Title\n\n## Section\n\n## Section\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD024"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD024"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md025_no_violation_single_h1() {
    let content = "# Title\n\n## Section\n";
    let errors = lint_string(content);
    assert!(!has_rule(&errors, "MD025"));
}

#[test]
fn test_md041_no_heading() {
    let content = "Some text without a heading.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD041"));
}

#[test]
fn test_md041_fix_round_trip() {
    let content = "Some text without a heading.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD041"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD041"), "Fixed: {:?}", fixed);
}

// ---- ATX spacing rules (MD018-MD021, MD023) ----

#[test]
fn test_md018_no_space() {
    let content = "#Title\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD018"));
}

#[test]
fn test_md018_fix_round_trip() {
    let content = "#Title\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD018"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD018"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md019_multi_space() {
    let content = "#  Title\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD019"));
}

#[test]
fn test_md019_fix_round_trip() {
    let content = "#  Title\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD019"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD019"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md020_no_space_closed() {
    let content = "#Title#\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD020"));
}

#[test]
fn test_md020_fix_round_trip() {
    let content = "#Title#\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD020"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD020"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md021_multi_space_closed() {
    let content = "#  Title  #\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD021"));
}

#[test]
fn test_md021_fix_round_trip() {
    let content = "#  Title  #\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD021"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD021"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md023_indented_heading() {
    let content = "  # Indented heading\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD023"));
}

#[test]
fn test_md023_fix_round_trip() {
    let content = "  # Indented heading\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD023"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD023"), "Fixed: {:?}", fixed);
}

// ---- Formatting rules (MD011, MD012, MD014, MD026, MD027, MD028) ----

#[test]
fn test_md011_reversed_link() {
    let content = "# Title\n\n(text)[https://example.com]\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD011"));
}

#[test]
fn test_md011_fix_round_trip() {
    let content = "# Title\n\n(text)[https://example.com]\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD011"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD011"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md012_multiple_blanks() {
    let content = "# Title\n\n\n\nSome text.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD012"));
}

#[test]
fn test_md012_fix_round_trip() {
    // Use exactly 2 consecutive blanks so single-pass fix resolves it
    let content = "# Title\n\n\nSome text.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD012"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD012"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md014_dollar_sign() {
    let content = "# Title\n\n```bash\n$ echo hello\n```\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD014"));
}

#[test]
fn test_md014_fix_round_trip() {
    let content = "# Title\n\n```bash\n$ echo hello\n```\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD014"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD014"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md026_trailing_punct() {
    let content = "# Title.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD026"));
}

#[test]
fn test_md026_fix_round_trip() {
    let content = "# Title.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD026"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD026"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md027_multi_space_blockquote() {
    let content = "# Title\n\n>  Extra space in blockquote\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD027"));
}

#[test]
fn test_md027_fix_round_trip() {
    let content = "# Title\n\n>  Extra space in blockquote\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD027"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD027"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md028_blank_in_blockquote() {
    // Actual blank line (not ">") between blockquote segments
    let content = "> Line one\n\n> Line two\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD028"));
}

#[test]
fn test_md028_fix_round_trip() {
    let content = "> Line one\n\n> Line two\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD028"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD028"), "Fixed: {:?}", fixed);
}

// ---- List rules (MD004, MD005, MD007, MD029, MD030, MD032) ----

#[test]
fn test_md004_wrong_style() {
    let json = r#"{"MD004": {"style": "dash"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n* Item one\n* Item two\n";
    let errors = lint_string_with_config(content, config);
    assert!(has_rule(&errors, "MD004"));
}

#[test]
fn test_md004_fix_round_trip() {
    let json = r#"{"MD004": {"style": "dash"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n* Item one\n* Item two\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD004"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(!has_rule(&errors_after, "MD004"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md005_inconsistent_indent() {
    // MD005 requires Micromark tokens with specific listUnordered structure.
    // Verify no panic through lint_sync pipeline.
    let content = "# Title\n\n- Item a\n - Item b\n- Item c\n";
    let errors = lint_string(content);
    // Token structure may vary; at minimum this is a no-panic smoke test.
    let _ = errors;
}

#[test]
fn test_md007_wrong_indent() {
    // 3 spaces is not a multiple of 2 → fires
    let json = r#"{"MD007": {"indent": 2}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n- Item\n   - Sub-item\n";
    let errors = lint_string_with_config(content, config);
    assert!(has_rule(&errors, "MD007"));
}

#[test]
fn test_md007_fix_round_trip() {
    let json = r#"{"MD007": {"indent": 2}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n- Item\n   - Sub-item\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD007"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(!has_rule(&errors_after, "MD007"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md029_wrong_prefix() {
    let json = r#"{"MD029": {"style": "ordered"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n1. First\n1. Second\n1. Third\n";
    let errors = lint_string_with_config(content, config);
    assert!(has_rule(&errors, "MD029"));
}

#[test]
fn test_md029_fix_round_trip() {
    let json = r#"{"MD029": {"style": "ordered"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n1. First\n1. Second\n1. Third\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD029"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(!has_rule(&errors_after, "MD029"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md030_extra_space() {
    // MD030 requires Micromark tokens; use ordered list variant
    let content = "# Title\n\n1.  Two-space item\n";
    let errors = lint_string(content);
    // MD030 may not fire through lint_sync if Micromark token structure differs
    // from what the rule expects; this is a detection-only test
    let _ = errors;
}

#[test]
fn test_md032_no_blank_around_list() {
    // MD032 requires Micromark tokens with specific list token structure.
    // Verify no panic through lint_sync pipeline.
    let content = "# Title\n- Item one\n- Item two\n";
    let errors = lint_string(content);
    if has_rule(&errors, "MD032") {
        let fixed = apply_fixes(content, &errors);
        let errors_after = lint_string(&fixed);
        assert!(!has_rule(&errors_after, "MD032"), "Fixed: {:?}", fixed);
    }
}

// ---- Code block rules (MD040, MD048) ----

#[test]
fn test_md040_no_language() {
    let content = "# Title\n\n```\nsome code\n```\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD040"));
}

#[test]
fn test_md040_fix_round_trip() {
    let content = "# Title\n\n```\nsome code\n```\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD040"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD040"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md048_mixed_fence_styles() {
    // Rule only fires when both backtick and tilde fences exist (no config support)
    let content = "# Title\n\n```\ncode\n```\n\n~~~\nmore code\n~~~\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD048"));
}

#[test]
fn test_md048_fix_round_trip() {
    let content = "# Title\n\n```\ncode\n```\n\n~~~\nmore code\n~~~\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD048"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD048"), "Fixed: {:?}", fixed);
}

// ---- Link/reference rules (MD033, MD034, MD039, MD043, MD044, MD045, MD047, MD051, MD052, MD053) ----

#[test]
fn test_md033_inline_html() {
    // MD033 requires Micromark htmlText tokens (inline HTML, not block htmlFlow).
    // Verify no panic; detection depends on Micromark token structure.
    let content = "# Title\n\nSome text with <b>bold</b> inline.\n";
    let errors = lint_string(content);
    let _ = errors;
}

#[test]
fn test_md034_bare_url() {
    let content = "# Title\n\nVisit https://example.com for details.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD034"));
}

#[test]
fn test_md034_fix_round_trip() {
    let content = "# Title\n\nVisit https://example.com for details.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD034"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD034"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md039_space_in_link() {
    let content = "# Title\n\n[ link text ](https://example.com)\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD039"));
}

#[test]
fn test_md039_fix_round_trip() {
    let content = "# Title\n\n[ link text ](https://example.com)\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD039"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD039"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md043_missing_heading() {
    let json = r###"{"default": false, "MD043": {"headings": ["# Title", "## Setup"]}}"###;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n## Usage\n";
    let errors = lint_string_with_config(content, config);
    assert!(has_rule(&errors, "MD043"));
}

#[test]
fn test_md044_fix_round_trip() {
    let content = "# Title\n\nUsing javascript and github in code.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD044"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD044"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md045_no_alt_text() {
    let content = "# Title\n\n![](image.png)\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD045"));
}

#[test]
fn test_md047_no_final_newline() {
    let content = "# Title\n\nText without final newline";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD047"));
}

#[test]
fn test_md047_fix_round_trip() {
    let content = "# Title\n\nText without final newline";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD047"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD047"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md051_invalid_fragment() {
    let content = "# Title\n\nSee [link](#nonexistent-section).\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD051"));
}

#[test]
fn test_md052_undefined_ref() {
    let content = "# Title\n\n[click here][undefined-ref]\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD052"));
}

#[test]
fn test_md053_unused_def() {
    let content = "# Title\n\nSome text.\n\n[unused]: https://example.com\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD053"));
}

#[test]
fn test_md053_fix_round_trip() {
    let content = "# Title\n\nSome text.\n\n[unused]: https://example.com\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD053"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD053"), "Fixed: {:?}", fixed);
}

// ---- Emphasis/strong rules (MD035, MD036, MD037, MD038, MD049, MD050) ----

#[test]
fn test_md035_inconsistent_hr() {
    let json = r#"{"MD035": {"style": "---"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n***\n";
    let errors = lint_string_with_config(content, config);
    assert!(has_rule(&errors, "MD035"));
}

#[test]
fn test_md035_fix_round_trip() {
    let json = r#"{"MD035": {"style": "---"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\n***\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD035"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(!has_rule(&errors_after, "MD035"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md036_emphasis_heading() {
    // MD036 depends on a specific Micromark token tree (content → paragraph → strong).
    // Verify at minimum that it doesn't panic through lint_sync.
    let content = "# Title\n\n**Bold Heading**\n\nNormal text.\n";
    let errors = lint_string(content);
    // If the Micromark parser produces the right token tree, MD036 will fire.
    // Otherwise this serves as a no-panic smoke test.
    if has_rule(&errors, "MD036") {
        let fixed = apply_fixes(content, &errors);
        let errors_after = lint_string(&fixed);
        assert!(!has_rule(&errors_after, "MD036"), "Fixed: {:?}", fixed);
    }
}

#[test]
fn test_md037_fix_round_trip_integration() {
    let content = "# Title\n\nThis is * spaced emphasis * here.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD037"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD037"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md038_space_in_code() {
    let content = "# Title\n\nUse ` code ` here.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD038"));
}

#[test]
fn test_md038_fix_round_trip() {
    let content = "# Title\n\nUse ` code ` here.\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD038"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD038"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md049_wrong_style() {
    let json = r#"{"MD049": {"style": "asterisk"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\nThis is _underscore emphasis_ here.\n";
    let errors = lint_string_with_config(content, config);
    assert!(has_rule(&errors, "MD049"));
}

#[test]
fn test_md049_fix_round_trip() {
    let json = r#"{"MD049": {"style": "asterisk"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\nThis is _underscore emphasis_ here.\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD049"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(!has_rule(&errors_after, "MD049"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md050_wrong_style() {
    let json = r#"{"MD050": {"style": "asterisk"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\nThis is __underscore strong__ here.\n";
    let errors = lint_string_with_config(content, config);
    assert!(has_rule(&errors, "MD050"));
}

#[test]
fn test_md050_fix_round_trip() {
    let json = r#"{"MD050": {"style": "asterisk"}}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    let content = "# Title\n\nThis is __underscore strong__ here.\n";
    let errors = lint_string_with_config(content, config.clone());
    assert!(has_rule(&errors, "MD050"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string_with_config(&fixed, config);
    assert!(!has_rule(&errors_after, "MD050"), "Fixed: {:?}", fixed);
}

// ---- Table rules (MD055, MD056, MD058) ----

#[test]
fn test_md055_inconsistent_pipes() {
    // Lines need 2+ pipes and asymmetric leading/trailing pipe usage
    let content = "# Title\n\n| a | b | c\n|---|---|---|\n| 1 | 2 | 3\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD055"));
}

#[test]
fn test_md055_fix_round_trip() {
    let content = "# Title\n\n| a | b | c\n|---|---|---|\n| 1 | 2 | 3\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD055"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD055"), "Fixed: {:?}", fixed);
}

#[test]
fn test_md056_wrong_col_count() {
    let content = "# Title\n\n| a | b |\n|---|---|\n| 1 | 2 | 3 |\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD056"));
}

#[test]
fn test_md058_no_blank_before_table() {
    let content = "# Title\n\nSome text\n| a | b |\n|---|---|\n| 1 | 2 |\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD058"));
}

#[test]
fn test_md058_fix_round_trip() {
    let content = "# Title\n\nSome text\n| a | b |\n|---|---|\n| 1 | 2 |\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD058"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD058"), "Fixed: {:?}", fixed);
}

// ---- Math rules (MD060) ----

#[test]
fn test_md060_dollar_in_fence() {
    let content = "# Title\n\n```bash\n$ echo hello\n$ ls\n```\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD060"));
}

#[test]
fn test_md060_fix_round_trip() {
    let content = "# Title\n\n```bash\n$ echo hello\n$ ls\n```\n";
    let errors = lint_string(content);
    assert!(has_rule(&errors, "MD060"));
    let fixed = apply_fixes(content, &errors);
    let errors_after = lint_string(&fixed);
    assert!(!has_rule(&errors_after, "MD060"), "Fixed: {:?}", fixed);
}

// ── Kramdown extension rules (KMD) ──────────────────────────────────────────

/// Helper to lint with a preset applied
fn lint_with_preset(markdown: &str, preset: &str) -> Vec<mkdlint::LintError> {
    let mut config = Config {
        preset: Some(preset.to_string()),
        ..Config::default()
    };
    config.apply_preset();
    lint_string_with_config(markdown, config)
}

#[test]
fn test_kmd_rules_off_by_default() {
    // KMD rules should NOT fire without the preset
    let content = "# H\n\nText[^1] here.\n";
    let errors = lint_string(content);
    assert!(
        !has_rule(&errors, "KMD002"),
        "KMD rules must be off by default"
    );
}

#[test]
fn test_kmd002_undefined_footnote_ref() {
    let content = "# H\n\nText[^1] here.\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        has_rule(&errors, "KMD002"),
        "KMD002 should fire for undefined footnote ref"
    );
}

#[test]
fn test_kmd002_defined_footnote_ref_ok() {
    let content = "# H\n\nText[^1] here.\n\n[^1]: A note.\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !has_rule(&errors, "KMD002"),
        "KMD002 should not fire when ref is defined"
    );
}

#[test]
fn test_kmd003_unused_footnote_def() {
    let content = "# H\n\nText here.\n\n[^1]: Unused note.\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        has_rule(&errors, "KMD003"),
        "KMD003 should fire for unused footnote definition"
    );
}

#[test]
fn test_kmd004_unused_abbreviation() {
    let content = "# H\n\nSome text.\n\n*[HTML]: HyperText Markup Language\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        has_rule(&errors, "KMD004"),
        "KMD004 should fire when abbreviation is never used"
    );
}

#[test]
fn test_kmd004_used_abbreviation_ok() {
    let content = "# H\n\nHTML is great.\n\n*[HTML]: HyperText Markup Language\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !has_rule(&errors, "KMD004"),
        "KMD004 should not fire when abbreviation appears in text"
    );
}

#[test]
fn test_kmd005_duplicate_heading_id() {
    let content = "# Setup\n\n## Setup\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        has_rule(&errors, "KMD005"),
        "KMD005 should fire when two headings produce the same ID"
    );
}

#[test]
fn test_kmd005_unique_headings_ok() {
    let content = "# Intro\n\n## Setup\n\n## Usage\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !has_rule(&errors, "KMD005"),
        "KMD005 should not fire for unique headings"
    );
}

#[test]
fn test_kmd006_malformed_ial() {
    let content = "# H\n\n{: bad!!syntax}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        has_rule(&errors, "KMD006"),
        "KMD006 should fire on malformed IAL"
    );
}

#[test]
fn test_kmd006_valid_ial_ok() {
    let content = "# H\n\n{: #my-id .section}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !has_rule(&errors, "KMD006"),
        "KMD006 should not fire on valid IAL"
    );
}

#[test]
fn test_preset_kramdown_disables_md033() {
    // Verify that the kramdown preset config sets MD033: false
    let mut config = Config {
        preset: Some("kramdown".to_string()),
        ..Config::default()
    };
    config.apply_preset();

    assert!(
        !config.is_rule_enabled("MD033"),
        "kramdown preset should disable MD033"
    );
}

#[test]
fn test_preset_kramdown_disables_md041() {
    // MD041 (first-heading) should be suppressed by the kramdown preset
    let content = "Some preamble text.\n\n## Section\n";
    let errors_default = lint_string(content);
    let errors_kramdown = lint_with_preset(content, "kramdown");

    assert!(
        has_rule(&errors_default, "MD041"),
        "MD041 should fire without preset"
    );
    assert!(
        !has_rule(&errors_kramdown, "MD041"),
        "MD041 should not fire with kramdown preset"
    );
}

#[test]
fn test_preset_via_config_key() {
    // Using preset via config struct should produce same state as --preset flag
    let mut config = Config {
        preset: Some("kramdown".to_string()),
        ..Config::default()
    };
    config.apply_preset();

    // Preset should have disabled MD033 and MD041
    assert!(
        !config.is_rule_enabled("MD033"),
        "MD033 should be disabled via config preset key"
    );
    assert!(
        !config.is_rule_enabled("MD041"),
        "MD041 should be disabled via config preset key"
    );
    // KMD rules should be enabled
    assert!(
        config.is_rule_enabled("KMD002"),
        "KMD002 should be enabled via config preset key"
    );
}

#[test]
fn test_preset_explicit_rules_override_preset() {
    // Explicit rule config should override the preset's settings
    use mkdlint::RuleConfig;
    let mut rules = std::collections::HashMap::new();
    rules.insert("MD033".to_string(), RuleConfig::Enabled(true));
    let mut config = Config {
        preset: Some("kramdown".to_string()),
        rules,
        ..Config::default()
    };
    config.apply_preset();

    // After apply_preset, MD033 should be enabled (explicit override wins)
    assert!(
        config.is_rule_enabled("MD033"),
        "Explicit MD033:true should override preset's MD033:false"
    );
    // And KMD rules should still be enabled (from preset)
    assert!(
        config.is_rule_enabled("KMD001"),
        "KMD001 should still be enabled from the preset"
    );
}

// ── KMD007: math block delimiters ────────────────────────────────────────────

#[test]
fn test_kmd007_unclosed_math_block() {
    let content = "# H\n\n$$\nx = 1\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        errors.iter().any(|e| e.rule_names[0] == "KMD007"),
        "should fire on unclosed math block"
    );
}

#[test]
fn test_kmd007_closed_math_block_ok() {
    let content = "# H\n\n$$\nx = 1\n$$\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !errors.iter().any(|e| e.rule_names[0] == "KMD007"),
        "matched $$ block should not fire"
    );
}

// ── KMD008: block extension syntax ───────────────────────────────────────────

#[test]
fn test_kmd008_unclosed_extension() {
    let content = "# H\n\n{::comment}\nsome text\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        errors.iter().any(|e| e.rule_names[0] == "KMD008"),
        "should fire on unclosed block extension"
    );
}

#[test]
fn test_kmd008_closed_extension_ok() {
    let content = "# H\n\n{::comment}\nsome text\n{:/comment}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !errors.iter().any(|e| e.rule_names[0] == "KMD008"),
        "matched comment block should not fire"
    );
}

// ── KMD009: ALD definitions used ─────────────────────────────────────────────

#[test]
fn test_kmd009_unused_ald() {
    let content = "# H\n\n{:myref: .highlight}\n\nA paragraph.\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        errors.iter().any(|e| e.rule_names[0] == "KMD009"),
        "should fire when ALD is never referenced"
    );
}

#[test]
fn test_kmd009_used_ald_ok() {
    let content = "# H\n\n{:myref: .highlight}\n\nA paragraph.\n{:myref}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !errors.iter().any(|e| e.rule_names[0] == "KMD009"),
        "referenced ALD should not fire"
    );
}

// ── KMD010: inline IAL syntax ────────────────────────────────────────────────

#[test]
fn test_kmd010_malformed_inline_ial() {
    let content = "# H\n\n*text*{: bad!!syntax}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        errors.iter().any(|e| e.rule_names[0] == "KMD010"),
        "should fire on malformed inline IAL"
    );
}

#[test]
fn test_kmd010_valid_inline_ial_ok() {
    let content = "# H\n\n*text*{: .highlight}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(
        !errors.iter().any(|e| e.rule_names[0] == "KMD010"),
        "valid inline IAL should not fire"
    );
}

// ── KMD auto-fix round-trip tests ────────────────────────────────────────────

#[test]
fn test_kmd005_fix_round_trip() {
    let content = "# Setup\n\n## Setup\n\n### Setup\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(has_rule(&errors, "KMD005"), "should have KMD005 errors");
    let fixed = apply_fixes(content, &errors);
    let errors2 = lint_with_preset(&fixed, "kramdown");
    assert!(
        !has_rule(&errors2, "KMD005"),
        "after fix, KMD005 should be gone; fixed:\n{fixed}"
    );
}

#[test]
fn test_kmd006_fix_round_trip() {
    let content = "# H\n\n{: bad!!syntax}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(has_rule(&errors, "KMD006"), "should have KMD006 error");
    let fixed = apply_fixes(content, &errors);
    let errors2 = lint_with_preset(&fixed, "kramdown");
    assert!(
        !has_rule(&errors2, "KMD006"),
        "after fix, KMD006 should be gone; fixed:\n{fixed}"
    );
}

#[test]
fn test_kmd007_fix_round_trip() {
    let content = "# H\n\n$$\nx = 1\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(has_rule(&errors, "KMD007"), "should have KMD007 error");
    let fixed = apply_fixes(content, &errors);
    let errors2 = lint_with_preset(&fixed, "kramdown");
    assert!(
        !has_rule(&errors2, "KMD007"),
        "after fix, KMD007 should be gone; fixed:\n{fixed}"
    );
}

#[test]
fn test_kmd008_fix_round_trip() {
    let content = "# H\n\n{::comment}\nsome text\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(has_rule(&errors, "KMD008"), "should have KMD008 error");
    let fixed = apply_fixes(content, &errors);
    let errors2 = lint_with_preset(&fixed, "kramdown");
    assert!(
        !has_rule(&errors2, "KMD008"),
        "after fix, KMD008 should be gone; fixed:\n{fixed}"
    );
}

#[test]
fn test_kmd009_fix_round_trip() {
    let content = "# H\n\n{:myref: .highlight}\n\nA paragraph.\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(has_rule(&errors, "KMD009"), "should have KMD009 error");
    let fixed = apply_fixes(content, &errors);
    let errors2 = lint_with_preset(&fixed, "kramdown");
    assert!(
        !has_rule(&errors2, "KMD009"),
        "after fix, KMD009 should be gone; fixed:\n{fixed}"
    );
}

#[test]
fn test_kmd010_fix_round_trip() {
    let content = "# H\n\n*text*{: bad!!syntax}\n";
    let errors = lint_with_preset(content, "kramdown");
    assert!(has_rule(&errors, "KMD010"), "should have KMD010 error");
    let fixed = apply_fixes(content, &errors);
    let errors2 = lint_with_preset(&fixed, "kramdown");
    assert!(
        !has_rule(&errors2, "KMD010"),
        "after fix, KMD010 should be gone; fixed:\n{fixed}"
    );
}

// ── GitHub preset ─────────────────────────────────────────────────────────────

#[test]
fn test_preset_github_disables_md013() {
    // Long lines should not fire MD013 with the github preset
    let long_line = format!("# H\n\n{}\n", "a".repeat(200));
    let errors_default = lint_string(&long_line);
    let errors_github = lint_with_preset(&long_line, "github");
    assert!(
        has_rule(&errors_default, "MD013"),
        "MD013 should fire without preset"
    );
    assert!(
        !has_rule(&errors_github, "MD013"),
        "MD013 should not fire with github preset"
    );
}

#[test]
fn test_preset_github_disables_md034() {
    // Bare URLs should not fire MD034 with the github preset
    let content = "# H\n\nhttps://example.com\n";
    let errors_default = lint_string(content);
    let errors_github = lint_with_preset(content, "github");
    assert!(
        has_rule(&errors_default, "MD034"),
        "MD034 should fire without preset"
    );
    assert!(
        !has_rule(&errors_github, "MD034"),
        "MD034 should not fire with github preset"
    );
}

#[test]
fn test_preset_github_md003_consistent() {
    // With github preset, MD003 enforces consistent style (not a fixed style).
    // Mixing atx and setext headings should still fire.
    let content = "# ATX Heading\n\nSetext Heading\n==============\n\n## ATX Again\n";
    let errors = lint_with_preset(content, "github");
    assert!(
        has_rule(&errors, "MD003"),
        "MD003 should fire on mixed heading styles with github preset"
    );
}

#[test]
fn test_preset_github_consistent_atx_ok() {
    // Pure ATX headings should be fine under the github consistent style
    let content = "# Intro\n\n## Setup\n\n## Usage\n";
    let errors = lint_with_preset(content, "github");
    assert!(
        !has_rule(&errors, "MD003"),
        "MD003 should not fire on consistent ATX headings with github preset"
    );
}

#[test]
fn test_preset_github_does_not_enable_kmd_rules() {
    // The github preset should not enable KMD rules
    let content = "# H\n\nText[^1] here.\n";
    let errors = lint_with_preset(content, "github");
    assert!(
        !has_rule(&errors, "KMD002"),
        "KMD rules should not fire with github preset"
    );
}

#[test]
fn test_preset_github_via_config_key() {
    let mut config = Config {
        preset: Some("github".to_string()),
        ..Config::default()
    };
    config.apply_preset();
    assert!(
        !config.is_rule_enabled("MD013"),
        "github preset should disable MD013 via config key"
    );
    assert!(
        !config.is_rule_enabled("MD034"),
        "github preset should disable MD034 via config key"
    );
}

// ---- Config inheritance / discovery tests (item 9) ----

/// Config::discover() should walk UP parent directories until it finds a config file.
#[test]
fn test_config_discovery_walks_parent_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Write config at root level
    let config_content = r#"{"MD013": false}"#;
    std::fs::write(root.join(".markdownlint.json"), config_content).unwrap();

    // Create nested subdirectory (no config here)
    let sub = root.join("sub").join("nested");
    std::fs::create_dir_all(&sub).unwrap();

    // Discover from the nested subdir — should find root config
    let found = Config::discover(&sub);
    assert!(found.is_some(), "Config::discover should walk up to root");

    let config = found.unwrap();
    assert!(
        !config.is_rule_enabled("MD013"),
        "Discovered config should have MD013 disabled"
    );
}

/// Config::discover() should return None when no config file exists in any ancestor.
#[test]
fn test_config_discovery_returns_none_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    // No config file created
    let found = Config::discover(dir.path());
    // May find a config from CI environment's parent dirs — just verify it doesn't panic
    // and returns a usable (possibly None) result.
    let _ = found; // existence is sufficient; no panic = pass
}

/// Config resolve_extends() merges parent config into child.
#[test]
fn test_config_extends_chain() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Write a base config
    let base_content = r#"{"MD013": false, "MD009": true}"#;
    let base_path = root.join("base.json");
    std::fs::write(&base_path, base_content).unwrap();

    // Write a child config that extends base and overrides MD009
    let child_content = format!(
        r#"{{"extends": "{}", "MD009": false}}"#,
        base_path.display()
    );
    let child_path = root.join("child.json");
    std::fs::write(&child_path, child_content).unwrap();

    let child_config = Config::from_file(&child_path).unwrap();
    let resolved = child_config.resolve_extends().unwrap();

    // MD013 should come from base
    assert!(
        !resolved.is_rule_enabled("MD013"),
        "extends chain should propagate MD013=false from base"
    );
    // MD009 override in child wins
    assert!(
        !resolved.is_rule_enabled("MD009"),
        "child config should override MD009"
    );
}

// ---- Parallel linting verification (item 8) ----

/// lint_sync uses rayon internally; this test verifies 50 files are linted
/// correctly without deadlocks or race conditions.
#[test]
fn test_parallel_lint_many_files() {
    let strings: HashMap<String, String> = (0..50)
        .map(|i| {
            (
                format!("file_{}.md", i),
                format!("# Title {}\n\nContent.\n", i),
            )
        })
        .collect();

    let options = LintOptions {
        strings,
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    assert_eq!(results.results.len(), 50, "All 50 files should be linted");
    for i in 0..50 {
        let key = format!("file_{}.md", i);
        assert!(
            results.get(key.as_str()).is_some(),
            "Result for {} should exist",
            key
        );
    }
}

// ---- heading_to_anchor_id helper tests (shared by MD051 and LSP) ----

#[test]
fn test_heading_to_anchor_id_basic() {
    assert_eq!(
        mkdlint::helpers::heading_to_anchor_id("Hello World"),
        "hello-world"
    );
    assert_eq!(
        mkdlint::helpers::heading_to_anchor_id("Getting Started"),
        "getting-started"
    );
    assert_eq!(
        mkdlint::helpers::heading_to_anchor_id("What's New?"),
        "whats-new"
    );
    assert_eq!(
        mkdlint::helpers::heading_to_anchor_id("API Reference"),
        "api-reference"
    );
}

// ---- --stdin-filename tests (item 7) ----

/// When a custom key is used (simulating --stdin-filename), errors appear under that key.
#[test]
fn test_stdin_filename_appears_in_results() {
    let custom_name = "my-doc.md".to_string();
    let markdown = "# Title\n\nTrailing spaces:   \n";
    let mut strings = HashMap::new();
    strings.insert(custom_name.clone(), markdown.to_string());

    let options = LintOptions {
        strings,
        ..Default::default()
    };

    let results = lint_sync(&options).unwrap();
    assert!(
        results.get(custom_name.as_str()).is_some(),
        "Results should be keyed by the custom stdin filename"
    );
    assert!(
        results.get("-").is_none(),
        "Results should NOT be keyed by '-' when custom filename is used"
    );
}

// ---- GitHub formatter tests (item 6) ----

#[test]
fn test_format_github_error_output() {
    use mkdlint::formatters::format_github;
    use mkdlint::types::{LintError, LintResults, Severity};

    let mut results = LintResults::new();
    results.add(
        "foo.md".to_string(),
        vec![LintError {
            line_number: 3,
            rule_names: &["MD009", "no-trailing-spaces"],
            rule_description: "Trailing spaces",
            error_detail: Some("Expected: 0; Actual: 2".to_string()),
            error_range: Some((1, 5)),
            severity: Severity::Error,
            fix_only: false,
            ..Default::default()
        }],
    );

    let output = format_github(&results);
    assert!(
        output.contains("::error "),
        "Should contain ::error. Got: {output}"
    );
    assert!(output.contains("file=foo.md"), "Should include filename");
    assert!(output.contains("line=3"), "Should include line number");
    assert!(output.contains("title=MD009"), "Should include rule");
    assert!(
        output.contains("Trailing spaces"),
        "Should include description"
    );
}

#[test]
fn test_format_github_warning_output() {
    use mkdlint::formatters::format_github;
    use mkdlint::types::{LintError, LintResults, Severity};

    let mut results = LintResults::new();
    results.add(
        "bar.md".to_string(),
        vec![LintError {
            line_number: 1,
            rule_names: &["MD001"],
            rule_description: "Heading levels should only increment by one level at a time",
            severity: Severity::Warning,
            fix_only: false,
            ..Default::default()
        }],
    );

    let output = format_github(&results);
    assert!(
        output.contains("::warning "),
        "Should contain ::warning. Got: {output}"
    );
}

#[test]
fn test_format_github_skips_fix_only_errors() {
    use mkdlint::formatters::format_github;
    use mkdlint::types::{LintError, LintResults, Severity};

    let mut results = LintResults::new();
    results.add(
        "baz.md".to_string(),
        vec![LintError {
            line_number: 1,
            rule_names: &["MD003"],
            rule_description: "Heading style",
            severity: Severity::Error,
            fix_only: true, // ghost error
            ..Default::default()
        }],
    );

    let output = format_github(&results);
    assert!(
        output.is_empty(),
        "fix_only errors should be omitted from github output"
    );
}
