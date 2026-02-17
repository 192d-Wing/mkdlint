//! Insta snapshot tests for mkdlint
//!
//! These tests lint fixture files and snapshot the error output so that
//! any regressions in rule behavior are immediately visible as snapshot diffs.

use mkdlint::{LintOptions, lint_sync};
use std::collections::HashMap;

/// Helper: lint a markdown string and return a deterministic text representation of the errors.
fn lint_snapshot(markdown: &str) -> String {
    let mut strings = HashMap::new();
    strings.insert("test.md".to_string(), markdown.to_string());
    let options = LintOptions {
        strings,
        ..Default::default()
    };
    let results = lint_sync(&options).unwrap();
    let errors = results.get("test.md").unwrap_or(&[]);

    let mut lines = Vec::new();
    for e in errors {
        let mut line = format!(
            "test.md:{}: {} {}",
            e.line_number,
            e.rule_names.join("/"),
            e.rule_description,
        );
        if let Some(detail) = &e.error_detail {
            line.push_str(&format!(" [{}]", detail));
        }
        if let Some(ctx) = &e.error_context {
            line.push_str(&format!(" [Context: \"{}\"]", ctx));
        }
        if let Some((col, len)) = e.error_range {
            line.push_str(&format!(" (col {}, len {})", col, len));
        }
        if e.fix_info.is_some() {
            line.push_str(" [fixable]");
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// Helper: lint a fixture file from the tests/fixtures directory.
fn lint_fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
    lint_snapshot(&content)
}

#[test]
fn snapshot_clean_file() {
    let output = lint_fixture("clean.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_heading_errors() {
    let output = lint_fixture("heading_errors.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_whitespace_errors() {
    let output = lint_fixture("whitespace_errors.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_link_errors() {
    let output = lint_fixture("link_errors.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_emphasis_errors() {
    let output = lint_fixture("emphasis_errors.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_fixable_errors() {
    let output = lint_fixture("fixable_errors.md");
    insta::assert_snapshot!(output);
}

// --- Inline markdown snapshot tests for specific rule behaviors ---

#[test]
fn snapshot_md009_trailing_spaces() {
    let output = lint_snapshot("# Title\n\nLine with spaces   \nClean line\nMore spaces  \n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md010_hard_tabs() {
    let output = lint_snapshot("# Title\n\n\tIndented with tab\n\t\tDouble tab\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md013_long_lines() {
    let long = "a".repeat(120);
    let md = format!("# Title\n\n{}\n", long);
    let output = lint_snapshot(&md);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md034_bare_urls() {
    let output = lint_snapshot(
        "# Title\n\nVisit http://example.com for info.\n\nAlso https://test.org/path is good.\n",
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md037_emphasis_spaces() {
    let output = lint_snapshot("# Title\n\nThis is * spaced emphasis * here.\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md044_proper_names() {
    let output = lint_snapshot("# Title\n\nUsing javascript and github in text.\n");
    insta::assert_snapshot!(output);
}

// --- New fixture-based snapshot tests ---

#[test]
fn snapshot_list_rules() {
    let output = lint_fixture("list_rules.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_code_block_rules() {
    let output = lint_fixture("code_block_rules.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_table_rules() {
    let output = lint_fixture("table_rules.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_math_rules() {
    let output = lint_fixture("math_rules.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_misc_rules() {
    let output = lint_fixture("misc_rules.md");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_heading_rules_extended() {
    let output = lint_fixture("heading_rules_extended.md");
    insta::assert_snapshot!(output);
}

// --- New inline snapshot tests for specific rule behaviors ---

#[test]
fn snapshot_md003_setext_vs_atx() {
    let output = lint_snapshot("Title\n=====\n\n## Section\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md011_reversed_link() {
    let output = lint_snapshot("# Title\n\n(text)[url]\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md018_no_space() {
    let output = lint_snapshot("#Title without space\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md024_duplicate_headings() {
    let output = lint_snapshot("# Title\n\n## Section\n\n## Section\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md036_emphasis_heading() {
    let output = lint_snapshot("# Title\n\n**Bold Heading**\n\nSome text.\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md047_missing_newline() {
    let output = lint_snapshot("# Title\n\nText without final newline");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md051_broken_fragment() {
    let output = lint_snapshot("# Title\n\n[link](#missing)\n");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_md060_dollar_in_fence() {
    let output = lint_snapshot("# Title\n\n```bash\n$ echo hello\n$ npm install\n```\n");
    insta::assert_snapshot!(output);
}
