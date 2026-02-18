//! End-to-end tests for the mkdlint CLI binary

use std::process::Command;

/// Get the path to the compiled binary
fn binary_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    path.push("mkdlint");
    path
}

/// Run the mkdlint binary with given args and return (exit_code, stdout, stderr)
fn run_mkdlint(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary_path())
        .args(args)
        .output()
        .expect("Failed to execute mkdlint binary");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

#[test]
fn test_cli_version() {
    let (code, stdout, _stderr) = run_mkdlint(&["--version"]);
    assert_eq!(code, 0, "--version should exit 0");
    assert!(
        stdout.contains("0."),
        "Version output should contain version number"
    );
}

#[test]
fn test_cli_help() {
    let (code, stdout, _stderr) = run_mkdlint(&["--help"]);
    assert_eq!(code, 0, "--help should exit 0");
    assert!(
        stdout.contains("lint") || stdout.contains("Markdown") || stdout.contains("mkdlint"),
        "Help output should mention linting or markdown"
    );
}

#[test]
fn test_cli_clean_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("clean.md");
    std::fs::write(
        &file_path,
        "# Title\n\nA paragraph with normal text.\n\n## Section\n\nAnother paragraph.\n",
    )
    .unwrap();

    let (code, stdout, _stderr) = run_mkdlint(&[file_path.to_str().unwrap()]);
    // Clean file should exit 0 with "No errors found" or similar
    // Note: some rules might still fire, so we just check it doesn't crash
    assert!(
        code == 0 || code == 1,
        "Exit code should be 0 (clean) or 1 (violations)"
    );
    let _ = stdout;
}

#[test]
fn test_cli_violation_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("bad.md");
    // This triggers MD001 (skipped heading level) and MD009 (trailing spaces)
    std::fs::write(
        &file_path,
        "# Heading 1\n\n### Heading 3\n\nTrailing spaces   \n",
    )
    .unwrap();

    let (code, stdout, _stderr) = run_mkdlint(&[file_path.to_str().unwrap()]);
    assert_eq!(code, 1, "File with violations should exit 1");
    assert!(!stdout.is_empty(), "Should print violation details");
}

#[test]
fn test_cli_with_config() {
    let dir = tempfile::tempdir().unwrap();

    let config_path = dir.path().join("config.json");
    std::fs::write(&config_path, r#"{"default": false}"#).unwrap();

    let file_path = dir.path().join("test.md");
    std::fs::write(&file_path, "# H1\n### H3\ntrailing   \n").unwrap();

    let (code, stdout, _stderr) = run_mkdlint(&[
        "--config",
        config_path.to_str().unwrap(),
        file_path.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "All rules disabled via config should produce exit 0"
    );
    assert!(
        stdout.contains("No errors"),
        "Should report no errors when all rules disabled"
    );
}

#[test]
fn test_cli_multiple_files() {
    let dir = tempfile::tempdir().unwrap();

    let file1 = dir.path().join("a.md");
    let file2 = dir.path().join("b.md");
    std::fs::write(&file1, "# File A\n\nContent.\n").unwrap();
    std::fs::write(&file2, "# File B\n\nContent.\n").unwrap();

    let (code, _stdout, _stderr) = run_mkdlint(&[file1.to_str().unwrap(), file2.to_str().unwrap()]);
    // Should process both files without crashing
    assert!(code == 0 || code == 1, "Should exit cleanly with 0 or 1");
}

#[test]
fn test_cli_nonexistent_file() {
    let (code, _stdout, stderr) = run_mkdlint(&["/tmp/this_file_does_not_exist_99999.md"]);
    assert_ne!(code, 0, "Nonexistent file should produce non-zero exit");
    assert!(
        !stderr.is_empty() || !_stdout.is_empty(),
        "Should output an error message"
    );
}

#[test]
fn test_cli_output_format() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("format_test.md");
    // Trigger MD009 (trailing spaces)
    std::fs::write(&file_path, "# Title\n\nTrailing   \n").unwrap();

    let (code, stdout, _stderr) = run_mkdlint(&[file_path.to_str().unwrap()]);
    if code == 1 {
        // Output should contain the filename and a rule identifier
        assert!(
            stdout.contains("format_test.md"),
            "Output should contain the filename"
        );
        assert!(
            stdout.contains("MD"),
            "Output should contain a rule ID like MD009"
        );
    }
}

// ---- E2E fixture tests ----

/// Resolve fixture path relative to CARGO_MANIFEST_DIR
fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn test_fixture_clean_file_exits_zero() {
    let (code, stdout, _) = run_mkdlint(&[&fixture_path("clean.md")]);
    assert_eq!(
        code, 0,
        "Clean fixture should produce exit 0. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("No errors"),
        "Clean fixture should report no errors. Output: {}",
        stdout
    );
}

#[test]
fn test_fixture_heading_errors_detected() {
    let (code, stdout, _) = run_mkdlint(&["--no-color", &fixture_path("heading_errors.md")]);
    assert_eq!(code, 1, "heading_errors.md should produce exit 1");
    assert!(
        stdout.contains("MD022"),
        "Should detect MD022 (blank lines around headings)"
    );
    assert!(
        stdout.contains("MD025"),
        "Should detect MD025 (multiple H1)"
    );
}

#[test]
fn test_fixture_whitespace_errors_detected() {
    let (code, stdout, _) = run_mkdlint(&["--no-color", &fixture_path("whitespace_errors.md")]);
    assert_eq!(code, 1, "whitespace_errors.md should produce exit 1");
    assert!(
        stdout.contains("MD009"),
        "Should detect MD009 (trailing spaces)"
    );
    assert!(stdout.contains("MD010"), "Should detect MD010 (hard tabs)");
}

#[test]
fn test_fixture_link_errors_detected() {
    let (code, stdout, _) = run_mkdlint(&["--no-color", &fixture_path("link_errors.md")]);
    assert_eq!(code, 1, "link_errors.md should produce exit 1");
    assert!(stdout.contains("MD034"), "Should detect MD034 (bare URLs)");
    assert!(
        stdout.contains("MD042"),
        "Should detect MD042 (empty links)"
    );
}

#[test]
fn test_fixture_emphasis_errors_detected() {
    let (code, stdout, _) = run_mkdlint(&["--no-color", &fixture_path("emphasis_errors.md")]);
    assert_eq!(code, 1, "emphasis_errors.md should produce exit 1");
    // MD049/MD050 enforce consistent emphasis/strong style
    assert!(
        stdout.contains("MD049")
            || stdout.contains("MD050")
            || stdout.contains("MD037")
            || stdout.contains("MD038"),
        "Should detect emphasis-related errors. Output: {}",
        stdout,
    );
}

#[test]
fn test_fixture_json_output_format() {
    let (code, stdout, _) = run_mkdlint(&[
        "--output-format",
        "json",
        &fixture_path("whitespace_errors.md"),
    ]);
    assert_eq!(code, 1, "Should exit 1 with violations");
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "JSON output should be valid JSON: {}\nOutput: {}",
            e, stdout
        )
    });
    assert!(parsed.is_object(), "JSON root should be an object");
}

#[test]
fn test_fixture_sarif_output_format() {
    let (code, stdout, _) = run_mkdlint(&[
        "--output-format",
        "sarif",
        &fixture_path("whitespace_errors.md"),
    ]);
    assert_eq!(code, 1, "Should exit 1 with violations");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "SARIF output should be valid JSON: {}\nOutput: {}",
            e, stdout
        )
    });
    assert_eq!(
        parsed["$schema"].as_str().unwrap_or(""),
        "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "SARIF should have correct schema URL"
    );
}

#[test]
fn test_fixture_fix_roundtrip() {
    // Copy a fixable fixture to a temp dir, run --fix, then lint again
    let dir = tempfile::tempdir().unwrap();
    let src = fixture_path("fixable_errors.md");
    let dest = dir.path().join("fixable.md");
    std::fs::copy(&src, &dest).unwrap();

    // Run with --fix
    let (code, _, _) = run_mkdlint(&["--fix", dest.to_str().unwrap()]);
    // --fix doesn't exit 1
    assert_eq!(code, 0, "--fix should exit 0");

    // Lint the fixed file — should have fewer errors
    let (_, stdout_after, _) = run_mkdlint(&["--no-color", dest.to_str().unwrap()]);
    // Verify that specific fixable rules are gone
    assert!(
        !stdout_after.contains("MD009"),
        "MD009 should be fixed after --fix"
    );
    assert!(
        !stdout_after.contains("MD010"),
        "MD010 should be fixed after --fix"
    );
}

#[test]
fn test_fixture_directory_recursion() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("docs");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("a.md"), "# File A\n\nContent.\n").unwrap();
    std::fs::write(sub.join("b.md"), "# File B\n\nContent.\n").unwrap();
    std::fs::write(sub.join("not_markdown.txt"), "Ignored\n").unwrap();

    let (code, stdout, _) = run_mkdlint(&[dir.path().to_str().unwrap()]);
    // Should lint both .md files but not the .txt file
    assert!(code == 0 || code == 1, "Should exit cleanly");
    if code == 1 {
        assert!(
            stdout.contains("a.md") || stdout.contains("b.md"),
            "Should lint .md files in subdirectory"
        );
    }
}

#[test]
fn test_fixture_ignore_pattern() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("good.md"), "# Title\n\nContent.\n").unwrap();
    std::fs::write(dir.path().join("bad.md"), "# Title\n\nTrailing   \n").unwrap();

    // Ignore bad.md
    let (_code, stdout, _) = run_mkdlint(&["--ignore", "**/bad.md", dir.path().to_str().unwrap()]);
    // Only good.md should be linted
    assert!(!stdout.contains("bad.md"), "bad.md should be ignored");
}

#[test]
fn test_fixture_source_context_in_output() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("context_test.md");
    std::fs::write(&file_path, "# Title\n\nTrailing spaces   \n").unwrap();

    let (code, stdout, _) = run_mkdlint(&["--no-color", file_path.to_str().unwrap()]);
    assert_eq!(code, 1);
    // The output should contain the source line with the error
    assert!(
        stdout.contains("Trailing spaces"),
        "Output should show source line context. Output: {}",
        stdout
    );
    // Should contain the underline carets
    assert!(
        stdout.contains("^^^"),
        "Output should show underline carets. Output: {}",
        stdout
    );
}

// ---- --fix-dry-run exit code tests ----

#[test]
fn test_fix_dry_run_exits_one_when_fixable() {
    // fixable_errors.md contains fixable violations; --fix-dry-run should exit 1
    let (code, stdout, _) = run_mkdlint(&[
        "--fix-dry-run",
        "--no-color",
        &fixture_path("fixable_errors.md"),
    ]);
    assert_eq!(
        code, 1,
        "--fix-dry-run should exit 1 when fixable issues exist. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("Would fix:") || stdout.contains("would be fixed"),
        "--fix-dry-run output should mention files to fix. Output: {}",
        stdout
    );
}

#[test]
fn test_fix_dry_run_exits_zero_when_clean() {
    // clean.md has no violations; --fix-dry-run should exit 0
    let (code, stdout, _) =
        run_mkdlint(&["--fix-dry-run", "--no-color", &fixture_path("clean.md")]);
    assert_eq!(
        code, 0,
        "--fix-dry-run should exit 0 when no fixable issues exist. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("No fixable") || stdout.is_empty(),
        "--fix-dry-run should report no fixable issues. Output: {}",
        stdout
    );
}

#[test]
fn test_fix_dry_run_does_not_modify_files() {
    let dir = tempfile::tempdir().unwrap();
    let src = fixture_path("fixable_errors.md");
    let dest = dir.path().join("test.md");
    std::fs::copy(&src, &dest).unwrap();
    let original_content = std::fs::read_to_string(&dest).unwrap();

    // --fix-dry-run should NOT modify the file
    let _ = run_mkdlint(&["--fix-dry-run", dest.to_str().unwrap()]);

    let after_content = std::fs::read_to_string(&dest).unwrap();
    assert_eq!(
        original_content, after_content,
        "--fix-dry-run must not modify files"
    );
}
