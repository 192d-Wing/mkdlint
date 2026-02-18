//! KMD005 - No duplicate heading IDs
//!
//! In Kramdown, each heading gets an ID either from an explicit IAL (`{#id}`)
//! or from an auto-generated slug. Duplicate IDs break anchor navigation and
//! are invalid HTML.
//!
//! Auto-slug algorithm (matches Kramdown): lowercase the heading text, replace
//! spaces with hyphens, strip all non-alphanumeric-or-hyphen characters.

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Matches ATX headings (with optional trailing IAL): `## Title {#custom-id}`
static ATX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.+?)(?:\s*\{[^}]*\})?\s*$").expect("valid regex"));

/// Matches an explicit `{#id}` attribute in an IAL or inline heading suffix
static EXPLICIT_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^}]*#([A-Za-z][\w-]*)[^}]*\}").expect("valid regex"));

/// Generate a Kramdown-style heading slug from heading text.
///
/// Algorithm: lowercase, keep alphanumeric + hyphens, replace spaces with `-`,
/// strip everything else, collapse multiple hyphens.
fn kramdown_slug(text: &str) -> String {
    // Strip any trailing IAL from the text first
    let text = if let Some(pos) = text.rfind('{') {
        if text[pos..].ends_with('}') {
            text[..pos].trim()
        } else {
            text
        }
    } else {
        text
    };

    let mut slug = String::with_capacity(text.len());
    let mut prev_hyphen = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            for c in ch.to_lowercase() {
                slug.push(c);
            }
            prev_hyphen = false;
        } else if (ch == ' ' || ch == '-') && !prev_hyphen {
            slug.push('-');
            prev_hyphen = true;
        }
        // All other chars are stripped
    }

    slug.trim_matches('-').to_string()
}

pub struct KMD005;

impl Rule for KMD005 {
    fn names(&self) -> &'static [&'static str] {
        &["KMD005", "no-duplicate-heading-ids"]
    }

    fn description(&self) -> &'static str {
        "Heading IDs must be unique within the document"
    }

    fn tags(&self) -> &[&'static str] {
        &["kramdown", "headings", "ids", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn is_enabled_by_default(&self) -> bool {
        false
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let lines = params.lines;

        // id → (first_line, occurrence_count); count starts at 1 for first occurrence
        let mut seen: HashMap<String, (usize, usize)> = HashMap::new();
        let mut in_code_block = false;
        // Track previous non-empty line for setext heading detection
        let mut prev_text: Option<(&str, usize)> = None; // (text, line_number)

        for (idx, line) in lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

            // Track code fences
            if crate::helpers::is_code_fence(trimmed) {
                in_code_block = !in_code_block;
                prev_text = None;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Detect setext heading underlines: === (h1) or --- (h2, ≥2 chars)
            let is_setext_h1 = !trimmed.is_empty() && trimmed.chars().all(|c| c == '=');
            let is_setext_h2 =
                trimmed.len() >= 2 && !trimmed.is_empty() && trimmed.chars().all(|c| c == '-');

            if (is_setext_h1 || is_setext_h2) && prev_text.is_some() {
                if let Some((heading_text, heading_line)) = prev_text.take() {
                    let explicit_cap = EXPLICIT_ID_RE.captures(heading_text);
                    let id = if let Some(ref cap) = explicit_cap {
                        cap[1].to_string()
                    } else {
                        kramdown_slug(heading_text)
                    };

                    if !id.is_empty() {
                        let entry = seen.entry(id.clone()).or_insert((heading_line, 0));
                        entry.1 += 1;
                        let (first_line, count) = *entry;
                        if count > 1 {
                            // Fix: append ` {#id-N}` to the text line (heading_line)
                            let new_id = format!("{id}-{count}");
                            let fix_text = format!(" {{#{new_id}}}");
                            // Column after last non-newline char on the heading text line
                            let text_line = lines[heading_line - 1];
                            let text_no_newline =
                                text_line.trim_end_matches('\n').trim_end_matches('\r');
                            let insert_col = text_no_newline.len() + 1;
                            errors.push(LintError {
                                line_number: heading_line,
                                rule_names: self.names(),
                                rule_description: self.description(),
                                error_detail: Some(format!(
                                    "Duplicate heading ID '{id}' (first defined on line {first_line})"
                                )),
                                severity: Severity::Error,
                                fix_only: false,
                                fix_info: Some(FixInfo {
                                    line_number: Some(heading_line),
                                    edit_column: Some(insert_col),
                                    delete_count: None,
                                    insert_text: Some(fix_text),
                                }),
                                ..Default::default()
                            });
                        }
                    }
                }
                prev_text = None;
                continue;
            }

            // ATX headings
            if let Some(cap) = ATX_RE.captures(trimmed) {
                let heading_text = cap[2].trim();

                // Determine the heading ID: explicit takes priority
                let id = if let Some(explicit) = EXPLICIT_ID_RE.captures(trimmed) {
                    explicit[1].to_string()
                } else {
                    kramdown_slug(heading_text)
                };

                if id.is_empty() {
                    prev_text = None;
                    continue;
                }

                let entry = seen.entry(id.clone()).or_insert((line_number, 0));
                entry.1 += 1;
                let (first_line, count) = *entry;
                if count > 1 {
                    let new_id = format!("{id}-{count}");
                    let fix_text = format!(" {{#{new_id}}}");
                    // Insert at end of heading content (before newline)
                    let line_no_newline = line.trim_end_matches('\n').trim_end_matches('\r');
                    let insert_col = line_no_newline.len() + 1;
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names(),
                        rule_description: self.description(),
                        error_detail: Some(format!(
                            "Duplicate heading ID '{id}' (first defined on line {first_line})"
                        )),
                        severity: Severity::Error,
                        fix_only: false,
                        fix_info: Some(FixInfo {
                            line_number: Some(line_number),
                            edit_column: Some(insert_col),
                            delete_count: None,
                            insert_text: Some(fix_text),
                        }),
                        ..Default::default()
                    });
                }
                prev_text = None;
                continue;
            }

            // Track previous non-empty line for setext detection
            if trimmed.is_empty() {
                prev_text = None;
            } else {
                prev_text = Some((trimmed, line_number));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleParams;
    use std::collections::HashMap;

    fn lint(content: &str) -> Vec<LintError> {
        let lines: Vec<&str> = content.split_inclusive('\n').collect();
        let rule = KMD005;
        rule.lint(&RuleParams {
            name: "test.md",
            version: "0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
            workspace_headings: None,
        })
    }

    #[test]
    fn test_kmd005_unique_headings_ok() {
        let errors = lint("# Intro\n\n## Setup\n\n## Usage\n");
        assert!(errors.is_empty(), "unique headings should not fire");
    }

    #[test]
    fn test_kmd005_duplicate_auto_slug() {
        let errors = lint("# Setup\n\n## Setup\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD005")),
            "should fire when two headings produce the same auto-slug"
        );
    }

    #[test]
    fn test_kmd005_explicit_id_duplicate() {
        let errors = lint("# Title {#intro}\n\n## Other {#intro}\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD005")),
            "should fire when two headings share an explicit ID"
        );
    }

    #[test]
    fn test_kmd005_kramdown_slug_generation() {
        assert_eq!(kramdown_slug("Hello World"), "hello-world");
        assert_eq!(kramdown_slug("Setup & Config!"), "setup-config");
        assert_eq!(kramdown_slug("  Leading spaces  "), "leading-spaces");
    }

    #[test]
    fn test_kmd005_setext_duplicate() {
        let errors = lint("Title\n=====\n\nTitle\n=====\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD005")),
            "should fire on duplicate setext headings"
        );
    }

    #[test]
    fn test_kmd005_setext_atx_collision() {
        // ATX # Title and setext Title\n----- produce the same slug
        let errors = lint("# Title\n\nTitle\n-----\n");
        assert!(
            errors
                .iter()
                .any(|e| e.rule_names.first() == Some(&"KMD005")),
            "should fire when setext and ATX heading share the same slug"
        );
    }

    #[test]
    fn test_kmd005_setext_thematic_break_ok() {
        // A bare --- with no preceding content line is a thematic break, not a heading
        let errors = lint("# Intro\n\n---\n\nParagraph\n");
        assert!(
            errors.is_empty(),
            "bare --- after blank line should not be treated as setext heading"
        );
    }

    #[test]
    fn test_kmd005_fix_info_present() {
        let errors = lint("# Setup\n\n## Setup\n");
        let err = errors
            .iter()
            .find(|e| e.rule_names.first() == Some(&"KMD005"))
            .unwrap();
        assert!(
            err.fix_info.is_some(),
            "duplicate heading should have fix_info"
        );
        let fix = err.fix_info.as_ref().unwrap();
        // Should insert " {#setup-2}" at the end of the heading line
        assert_eq!(fix.insert_text.as_deref(), Some(" {#setup-2}"));
        assert!(fix.delete_count.is_none());
    }

    #[test]
    fn test_kmd005_fix_round_trip() {
        use crate::lint::apply_fixes;
        let content = "# Setup\n\n## Setup\n";
        let errors = lint(content);
        let fixed = apply_fixes(content, &errors);
        // After fix, re-linting should produce no KMD005 errors
        let errors2 = lint(&fixed);
        assert!(
            errors2
                .iter()
                .all(|e| e.rule_names.first() != Some(&"KMD005")),
            "after fix, no KMD005 errors expected; got: {errors2:?}"
        );
    }

    #[test]
    fn test_kmd005_fix_triple_duplicate() {
        use crate::lint::apply_fixes;
        let content = "# Intro\n\n## Intro\n\n### Intro\n";
        let errors = lint(content);
        assert_eq!(errors.len(), 2, "two duplicate errors expected");
        // Check suffixes
        let texts: Vec<_> = errors
            .iter()
            .filter_map(|e| e.fix_info.as_ref())
            .filter_map(|f| f.insert_text.as_deref())
            .collect();
        assert!(texts.contains(&" {#intro-2}"), "second gets -2");
        assert!(texts.contains(&" {#intro-3}"), "third gets -3");
        let fixed = apply_fixes(content, &errors);
        let errors2 = lint(&fixed);
        assert!(
            errors2
                .iter()
                .all(|e| e.rule_names.first() != Some(&"KMD005")),
            "after fix, no KMD005 errors; got: {errors2:?}"
        );
    }
}
