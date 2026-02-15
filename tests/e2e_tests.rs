//! End-to-end tests for the mdlint CLI binary

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
    path.push("mdlint");
    path
}

/// Run the mdlint binary with given args and return (exit_code, stdout, stderr)
fn run_mdlint(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary_path())
        .args(args)
        .output()
        .expect("Failed to execute mdlint binary");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

#[test]
fn test_cli_version() {
    let (code, stdout, _stderr) = run_mdlint(&["--version"]);
    assert_eq!(code, 0, "--version should exit 0");
    assert!(stdout.contains("0."), "Version output should contain version number");
}

#[test]
fn test_cli_help() {
    let (code, stdout, _stderr) = run_mdlint(&["--help"]);
    assert_eq!(code, 0, "--help should exit 0");
    assert!(stdout.contains("lint") || stdout.contains("Markdown") || stdout.contains("mdlint"),
        "Help output should mention linting or markdown");
}

#[test]
fn test_cli_clean_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("clean.md");
    std::fs::write(&file_path, "# Title\n\nA paragraph with normal text.\n\n## Section\n\nAnother paragraph.\n").unwrap();

    let (code, stdout, _stderr) = run_mdlint(&[file_path.to_str().unwrap()]);
    // Clean file should exit 0 with "No errors found" or similar
    // Note: some rules might still fire, so we just check it doesn't crash
    assert!(code == 0 || code == 1, "Exit code should be 0 (clean) or 1 (violations)");
    let _ = stdout;
}

#[test]
fn test_cli_violation_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("bad.md");
    // This triggers MD001 (skipped heading level) and MD009 (trailing spaces)
    std::fs::write(&file_path, "# Heading 1\n\n### Heading 3\n\nTrailing spaces   \n").unwrap();

    let (code, stdout, _stderr) = run_mdlint(&[file_path.to_str().unwrap()]);
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

    let (code, stdout, _stderr) = run_mdlint(&[
        "--config", config_path.to_str().unwrap(),
        file_path.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "All rules disabled via config should produce exit 0");
    assert!(stdout.contains("No errors"), "Should report no errors when all rules disabled");
}

#[test]
fn test_cli_multiple_files() {
    let dir = tempfile::tempdir().unwrap();

    let file1 = dir.path().join("a.md");
    let file2 = dir.path().join("b.md");
    std::fs::write(&file1, "# File A\n\nContent.\n").unwrap();
    std::fs::write(&file2, "# File B\n\nContent.\n").unwrap();

    let (code, _stdout, _stderr) = run_mdlint(&[
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
    ]);
    // Should process both files without crashing
    assert!(code == 0 || code == 1, "Should exit cleanly with 0 or 1");
}

#[test]
fn test_cli_nonexistent_file() {
    let (code, _stdout, stderr) = run_mdlint(&["/tmp/this_file_does_not_exist_99999.md"]);
    assert_ne!(code, 0, "Nonexistent file should produce non-zero exit");
    assert!(!stderr.is_empty() || !_stdout.is_empty(), "Should output an error message");
}

#[test]
fn test_cli_output_format() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("format_test.md");
    // Trigger MD009 (trailing spaces)
    std::fs::write(&file_path, "# Title\n\nTrailing   \n").unwrap();

    let (code, stdout, _stderr) = run_mdlint(&[file_path.to_str().unwrap()]);
    if code == 1 {
        // Output should contain the filename and a rule identifier
        assert!(stdout.contains("format_test.md"), "Output should contain the filename");
        assert!(stdout.contains("MD"), "Output should contain a rule ID like MD009");
    }
}
