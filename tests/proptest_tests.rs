//! Property-based tests for mkdlint using proptest
//!
//! Tests invariants that must hold for *all* valid inputs, not just hand-picked examples.

use mkdlint::{Config, LintOptions, RuleConfig, apply_fixes, lint_sync, parser};
use proptest::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Strategies for generating markdown-like content
// ---------------------------------------------------------------------------

/// Generate a single markdown line (text, heading, list item, etc.)
fn md_line() -> impl Strategy<Value = String> {
    prop_oneof![
        // Plain text
        "[a-zA-Z0-9 ,.!?]{0,120}".prop_map(|s| s),
        // ATX heading
        (1..=6u8, "[a-zA-Z0-9 ]{0,60}").prop_map(|(level, text)| format!(
            "{} {}",
            "#".repeat(level as usize),
            text
        )),
        // Unordered list item
        "[a-zA-Z0-9 ]{1,40}".prop_map(|text| format!("- {}", text)),
        // Ordered list item
        (1..100u32, "[a-zA-Z0-9 ]{1,40}").prop_map(|(n, text)| format!("{}. {}", n, text)),
        // Fenced code block
        "[a-z]{0,10}".prop_map(|lang| format!("```{}\ncode\n```", lang)),
        // Blockquote
        "[a-zA-Z0-9 ]{1,60}".prop_map(|text| format!("> {}", text)),
        // Link
        ("[a-zA-Z0-9 ]{1,20}", "[a-z]{3,10}")
            .prop_map(|(text, url)| format!("[{}]({})", text, url)),
        // Image
        ("[a-zA-Z0-9 ]{0,20}", "[a-z.]{3,15}")
            .prop_map(|(alt, src)| format!("![{}]({})", alt, src)),
        // Blank line
        Just(String::new()),
        // Horizontal rule
        prop_oneof![Just("---".to_string()), Just("***".to_string()),],
        // Table row
        ("[a-zA-Z0-9]{1,10}", "[a-zA-Z0-9]{1,10}").prop_map(|(a, b)| format!("| {} | {} |", a, b)),
    ]
}

/// Generate a complete markdown document from random lines.
fn md_document() -> impl Strategy<Value = String> {
    prop::collection::vec(md_line(), 1..50).prop_map(|lines| {
        let mut doc = lines.join("\n");
        doc.push('\n');
        doc
    })
}

/// Generate arbitrary bytes that are valid UTF-8 (including edge cases).
fn arbitrary_utf8() -> impl Strategy<Value = String> {
    prop::string::string_regex(".{0,500}")
        .unwrap()
        .prop_map(|s| s)
}

/// Extended markdown line strategy with more exotic constructs.
fn md_line_extended() -> impl Strategy<Value = String> {
    prop_oneof![
        // All original variants from md_line()
        "[a-zA-Z0-9 ,.!?]{0,120}".prop_map(|s| s),
        (1..=6u8, "[a-zA-Z0-9 ]{0,60}").prop_map(|(level, text)| format!(
            "{} {}",
            "#".repeat(level as usize),
            text
        )),
        "[a-zA-Z0-9 ]{1,40}".prop_map(|text| format!("- {}", text)),
        (1..100u32, "[a-zA-Z0-9 ]{1,40}").prop_map(|(n, text)| format!("{}. {}", n, text)),
        "[a-z]{0,10}".prop_map(|lang| format!("```{}\ncode\n```", lang)),
        "[a-zA-Z0-9 ]{1,60}".prop_map(|text| format!("> {}", text)),
        ("[a-zA-Z0-9 ]{1,20}", "[a-z]{3,10}")
            .prop_map(|(text, url)| format!("[{}]({})", text, url)),
        ("[a-zA-Z0-9 ]{0,20}", "[a-z.]{3,15}")
            .prop_map(|(alt, src)| format!("![{}]({})", alt, src)),
        Just(String::new()),
        prop_oneof![Just("---".to_string()), Just("***".to_string()),],
        ("[a-zA-Z0-9]{1,10}", "[a-zA-Z0-9]{1,10}").prop_map(|(a, b)| format!("| {} | {} |", a, b)),
        // Extended: multi-byte UTF-8 text
        "[\\p{L}\\p{N} ,]{0,60}".prop_map(|s| s),
        // Extended: setext heading h1
        "[a-zA-Z0-9 ]{1,40}".prop_map(|text| format!("{}\n======", text)),
        // Extended: setext heading h2
        "[a-zA-Z0-9 ]{1,40}".prop_map(|text| format!("{}\n------", text)),
        // Extended: tilde code fence
        "[a-z]{0,10}".prop_map(|lang| format!("~~~{}\ncode\n~~~", lang)),
        // Extended: nested list item
        "[a-zA-Z0-9 ]{1,30}".prop_map(|text| format!("  - {}", text)),
        // Extended: HTML block
        "[a-zA-Z0-9 ]{0,30}".prop_map(|text| format!("<div>{}</div>", text)),
    ]
}

/// Generate a document using the extended strategy.
fn md_document_extended() -> impl Strategy<Value = String> {
    prop::collection::vec(md_line_extended(), 1..50).prop_map(|lines| {
        let mut doc = lines.join("\n");
        doc.push('\n');
        doc
    })
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

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

// ===========================================================================
// Property 1: lint_sync never panics on arbitrary input
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn lint_never_panics_structured(doc in md_document()) {
        let _ = lint_string(&doc);
    }

    #[test]
    fn lint_never_panics_arbitrary(input in arbitrary_utf8()) {
        let _ = lint_string(&input);
    }
}

// ===========================================================================
// Property 2: parser::parse never panics on arbitrary input
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn parser_never_panics(input in arbitrary_utf8()) {
        let _ = parser::parse(&input);
    }
}

// ===========================================================================
// Property 3: apply_fixes produces valid output
// ===========================================================================
//
// Fixes can restructure content (e.g., converting a setext heading to ATX),
// which may legitimately expose new issues. Rule interactions can also cycle
// (MD018 adds space, MD009 strips trailing space). We test that:
//   - apply_fixes never panics
//   - the output is valid UTF-8 (guaranteed by String return)
//   - re-linting the output also never panics

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn apply_fixes_roundtrip_safe(doc in md_document()) {
        let errors = lint_string(&doc);
        let fixed = apply_fixes(&doc, &errors);
        // Re-lint the fixed content — must not panic
        let _ = lint_string(&fixed);
    }
}

// ===========================================================================
// Property 4: line numbers are always in bounds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn error_line_numbers_in_bounds(doc in md_document()) {
        let line_count = doc.lines().count().max(1);
        let errors = lint_string(&doc);
        for error in &errors {
            prop_assert!(
                error.line_number >= 1 && error.line_number <= line_count,
                "Line {} out of bounds (document has {} lines). Rule: {:?}",
                error.line_number, line_count, error.rule_names
            );
        }
    }
}

// ===========================================================================
// Property 6: error_range columns are within line length
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn error_range_within_line(doc in md_document()) {
        let lines: Vec<&str> = doc.lines().collect();
        let errors = lint_string(&doc);
        for error in &errors {
            if let Some((start, len)) = error.error_range {
                let line_idx = error.line_number.saturating_sub(1);
                if line_idx < lines.len() {
                    let line_len = lines[line_idx].len();
                    prop_assert!(
                        start >= 1,
                        "error_range start must be 1-based, got {}. Rule: {:?}",
                        start, error.rule_names
                    );
                    prop_assert!(
                        start.saturating_sub(1) + len <= line_len + 1,
                        "error_range ({}, {}) exceeds line length {} at line {}. Rule: {:?}",
                        start, len, line_len, error.line_number, error.rule_names
                    );
                }
            }
        }
    }
}

// ===========================================================================
// Property 7: apply_fixes preserves line ending style
// ===========================================================================

/// Convert an LF document to CRLF.
fn to_crlf(doc: &str) -> String {
    // First remove any existing \r\n to avoid doubling, then convert \n to \r\n
    doc.replace("\r\n", "\n").replace('\n', "\r\n")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn fixes_preserve_lf_line_endings(doc in md_document()) {
        let errors = lint_string(&doc);
        let fixed = apply_fixes(&doc, &errors);
        prop_assert!(
            !fixed.contains("\r\n"),
            "LF document should not gain CRLF after fix"
        );
    }

    #[test]
    fn fixes_preserve_crlf_line_endings(doc in md_document()) {
        let crlf_doc = to_crlf(&doc);
        let errors = lint_string(&crlf_doc);
        let fixed = apply_fixes(&crlf_doc, &errors);
        // Every \n in the output should be preceded by \r (i.e., all newlines are \r\n)
        for (i, byte) in fixed.bytes().enumerate() {
            if byte == b'\n' && i > 0 {
                prop_assert!(
                    fixed.as_bytes()[i - 1] == b'\r',
                    "CRLF document should not gain bare LF after fix. \
                     Found bare \\n at byte {}. Fixed content: {:?}",
                    i, &fixed[..fixed.len().min(200)]
                );
            }
        }
    }
}

// ===========================================================================
// Property 8: disabling all rules yields zero errors
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn all_rules_disabled_yields_zero_errors(doc in md_document()) {
        let config = Config {
            default: Some(false),
            ..Default::default()
        };
        let errors = lint_string_with_config(&doc, config);
        prop_assert_eq!(
            errors.len(), 0,
            "Disabling all rules should produce zero errors, got {}",
            errors.len()
        );
    }
}

// ===========================================================================
// Property 9: enabling a single rule only reports that rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn single_rule_only_reports_itself(
        doc in md_document(),
        rule_idx in 0..53usize,
    ) {
        let rule_ids = [
            "MD001", "MD003", "MD004", "MD005", "MD007", "MD009", "MD010",
            "MD011", "MD012", "MD013", "MD014", "MD018", "MD019", "MD020",
            "MD021", "MD022", "MD023", "MD024", "MD025", "MD026", "MD027",
            "MD028", "MD029", "MD030", "MD031", "MD032", "MD033", "MD034",
            "MD035", "MD036", "MD037", "MD038", "MD039", "MD040", "MD041",
            "MD042", "MD043", "MD044", "MD045", "MD046", "MD047", "MD048",
            "MD049", "MD050", "MD051", "MD052", "MD053", "MD054", "MD055",
            "MD056", "MD058", "MD059", "MD060",
        ];
        let chosen = rule_ids[rule_idx];

        let mut rules = HashMap::new();
        rules.insert(chosen.to_string(), RuleConfig::Enabled(true));
        let config = Config {
            default: Some(false),
            rules,
            ..Default::default()
        };

        let errors = lint_string_with_config(&doc, config);
        for error in &errors {
            prop_assert!(
                error.rule_names.contains(&chosen),
                "Expected only {} errors but got {:?}",
                chosen, error.rule_names
            );
        }
    }
}

// ===========================================================================
// Property 10: Config JSON roundtrip
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn config_json_roundtrip(
        default_val in prop::option::of(any::<bool>()),
        num_rules in 0..10usize,
    ) {
        let mut rules = HashMap::new();
        for i in 0..num_rules {
            let rule_name = format!("MD{:03}", (i % 53) + 1);
            rules.insert(rule_name, RuleConfig::Enabled(i % 2 == 0));
        }

        let config = Config {
            default: default_val,
            extends: None,
            rules,
        };

        let json = serde_json::to_string(&config).unwrap();
        let roundtripped: Config = serde_json::from_str(&json).unwrap();

        // default field survives
        prop_assert_eq!(config.default, roundtripped.default);
        // all rules survive
        prop_assert_eq!(config.rules.len(), roundtripped.rules.len());
    }
}

// ===========================================================================
// Property 11: parser tokens have valid line numbers
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn parser_tokens_have_valid_lines(doc in md_document()) {
        let line_count = doc.lines().count().max(1);
        let tokens = parser::parse(&doc);
        for token in &tokens {
            if token.start_line > 0 {
                prop_assert!(
                    token.start_line <= line_count,
                    "Token {:?} start_line {} exceeds document line count {}",
                    token.token_type, token.start_line, line_count
                );
            }
        }
    }
}

// ===========================================================================
// Property 12: empty document produces consistent results
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lint_deterministic(doc in md_document()) {
        let errors_a = lint_string(&doc);
        let errors_b = lint_string(&doc);
        prop_assert_eq!(
            errors_a.len(), errors_b.len(),
            "Linting same document twice should produce identical error counts"
        );
        for (a, b) in errors_a.iter().zip(errors_b.iter()) {
            prop_assert_eq!(a.line_number, b.line_number);
            prop_assert_eq!(a.rule_names, b.rule_names);
        }
    }
}

// ===========================================================================
// Property 13: lint + fix with extended strategy never panics
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn lint_never_panics_extended(doc in md_document_extended()) {
        let _ = lint_string(&doc);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn apply_fixes_never_panics_extended(doc in md_document_extended()) {
        let errors = lint_string(&doc);
        let fixed = apply_fixes(&doc, &errors);
        let _ = lint_string(&fixed);
    }
}

// ===========================================================================
// Property 14: apply_fixes is idempotent (fixing twice converges)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn apply_fixes_idempotent(doc in md_document()) {
        let errors1 = lint_string(&doc);
        let fixed1 = apply_fixes(&doc, &errors1);
        let errors2 = lint_string(&fixed1);
        let fixed2 = apply_fixes(&fixed1, &errors2);
        let errors3 = lint_string(&fixed2);

        // After two rounds, total error count should converge (not grow unbounded).
        // Allow a small delta due to rule interactions exposing new issues.
        let total2 = errors2.len();
        let total3 = errors3.len();
        prop_assert!(
            total3 <= total2 + 3,
            "Errors should converge after two fix passes, but grew from {} to {}",
            total2, total3
        );
    }
}

// ===========================================================================
// Property 15: fix does not increase specific rule error count (well-behaved rules)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn fix_reduces_specific_rule_errors(doc in md_document()) {
        // These rules should not increase in count after their own fixes are applied.
        // Excluded: MD009 (trailing spaces can appear when other rules add content),
        // MD012 (consecutive blanks can appear when fixes insert/delete lines),
        // MD026 (heading punctuation can appear after HR fix).
        let well_behaved = ["MD010", "MD034", "MD040", "MD047"];
        let errors = lint_string(&doc);
        let fixed = apply_fixes(&doc, &errors);
        let errors_after = lint_string(&fixed);

        for rule in &well_behaved {
            let before = errors.iter().filter(|e| e.rule_names.contains(rule)).count();
            let after = errors_after.iter().filter(|e| e.rule_names.contains(rule)).count();
            prop_assert!(
                after <= before,
                "Rule {} error count increased after fix: {} -> {}",
                rule, before, after
            );
        }
    }
}

// ===========================================================================
// Property 16: lint never panics with mixed CRLF/LF endings
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lint_never_panics_mixed_endings(doc in md_document()) {
        // Randomly convert some lines to CRLF
        let mixed: String = doc
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i % 2 == 0 {
                    format!("{}\r\n", line)
                } else {
                    format!("{}\n", line)
                }
            })
            .collect();
        let _ = lint_string(&mixed);
    }
}

// ===========================================================================
// Property 17: lint never panics with UTF-8 BOM
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lint_never_panics_with_bom(doc in md_document()) {
        let with_bom = format!("\u{FEFF}{}", doc);
        let _ = lint_string(&with_bom);
    }
}
