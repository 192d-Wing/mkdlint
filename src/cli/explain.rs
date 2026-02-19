//! `--explain <RULE>` handler — print per-rule documentation
//!
//! Renders embedded Markdown docs with terminal-aware formatting:
//! word wrapping, inline bold/code styling, and pager support.

use colored::Colorize;
use regex::Regex;
use std::io::Write;
use std::sync::LazyLock;

/// Regex for `**bold**` inline formatting.
static BOLD_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());

/// Regex for `` `code` `` inline formatting.
static CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`]+)`").unwrap());

/// Mapping of canonical rule ID (uppercase) to embedded doc content.
/// All docs are embedded at compile time via include_str!().
fn get_rule_doc(canonical: &str) -> Option<&'static str> {
    match canonical {
        "MD001" => Some(include_str!("../../docs/rules/md001.md")),
        "MD003" => Some(include_str!("../../docs/rules/md003.md")),
        "MD004" => Some(include_str!("../../docs/rules/md004.md")),
        "MD005" => Some(include_str!("../../docs/rules/md005.md")),
        "MD007" => Some(include_str!("../../docs/rules/md007.md")),
        "MD009" => Some(include_str!("../../docs/rules/md009.md")),
        "MD010" => Some(include_str!("../../docs/rules/md010.md")),
        "MD011" => Some(include_str!("../../docs/rules/md011.md")),
        "MD012" => Some(include_str!("../../docs/rules/md012.md")),
        "MD013" => Some(include_str!("../../docs/rules/md013.md")),
        "MD014" => Some(include_str!("../../docs/rules/md014.md")),
        "MD018" => Some(include_str!("../../docs/rules/md018.md")),
        "MD019" => Some(include_str!("../../docs/rules/md019.md")),
        "MD020" => Some(include_str!("../../docs/rules/md020.md")),
        "MD021" => Some(include_str!("../../docs/rules/md021.md")),
        "MD022" => Some(include_str!("../../docs/rules/md022.md")),
        "MD023" => Some(include_str!("../../docs/rules/md023.md")),
        "MD024" => Some(include_str!("../../docs/rules/md024.md")),
        "MD025" => Some(include_str!("../../docs/rules/md025.md")),
        "MD026" => Some(include_str!("../../docs/rules/md026.md")),
        "MD027" => Some(include_str!("../../docs/rules/md027.md")),
        "MD028" => Some(include_str!("../../docs/rules/md028.md")),
        "MD029" => Some(include_str!("../../docs/rules/md029.md")),
        "MD030" => Some(include_str!("../../docs/rules/md030.md")),
        "MD031" => Some(include_str!("../../docs/rules/md031.md")),
        "MD032" => Some(include_str!("../../docs/rules/md032.md")),
        "MD033" => Some(include_str!("../../docs/rules/md033.md")),
        "MD034" => Some(include_str!("../../docs/rules/md034.md")),
        "MD035" => Some(include_str!("../../docs/rules/md035.md")),
        "MD036" => Some(include_str!("../../docs/rules/md036.md")),
        "MD037" => Some(include_str!("../../docs/rules/md037.md")),
        "MD038" => Some(include_str!("../../docs/rules/md038.md")),
        "MD039" => Some(include_str!("../../docs/rules/md039.md")),
        "MD040" => Some(include_str!("../../docs/rules/md040.md")),
        "MD041" => Some(include_str!("../../docs/rules/md041.md")),
        "MD042" => Some(include_str!("../../docs/rules/md042.md")),
        "MD043" => Some(include_str!("../../docs/rules/md043.md")),
        "MD044" => Some(include_str!("../../docs/rules/md044.md")),
        "MD045" => Some(include_str!("../../docs/rules/md045.md")),
        "MD046" => Some(include_str!("../../docs/rules/md046.md")),
        "MD047" => Some(include_str!("../../docs/rules/md047.md")),
        "MD048" => Some(include_str!("../../docs/rules/md048.md")),
        "MD049" => Some(include_str!("../../docs/rules/md049.md")),
        "MD050" => Some(include_str!("../../docs/rules/md050.md")),
        "MD051" => Some(include_str!("../../docs/rules/md051.md")),
        "MD052" => Some(include_str!("../../docs/rules/md052.md")),
        "MD053" => Some(include_str!("../../docs/rules/md053.md")),
        "MD054" => Some(include_str!("../../docs/rules/md054.md")),
        "MD055" => Some(include_str!("../../docs/rules/md055.md")),
        "MD056" => Some(include_str!("../../docs/rules/md056.md")),
        "MD058" => Some(include_str!("../../docs/rules/md058.md")),
        "MD059" => Some(include_str!("../../docs/rules/md059.md")),
        "MD060" => Some(include_str!("../../docs/rules/md060.md")),
        "KMD001" => Some(include_str!("../../docs/rules/kmd001.md")),
        "KMD002" => Some(include_str!("../../docs/rules/kmd002.md")),
        "KMD003" => Some(include_str!("../../docs/rules/kmd003.md")),
        "KMD004" => Some(include_str!("../../docs/rules/kmd004.md")),
        "KMD005" => Some(include_str!("../../docs/rules/kmd005.md")),
        "KMD006" => Some(include_str!("../../docs/rules/kmd006.md")),
        "KMD007" => Some(include_str!("../../docs/rules/kmd007.md")),
        "KMD008" => Some(include_str!("../../docs/rules/kmd008.md")),
        "KMD009" => Some(include_str!("../../docs/rules/kmd009.md")),
        "KMD010" => Some(include_str!("../../docs/rules/kmd010.md")),
        "KMD011" => Some(include_str!("../../docs/rules/kmd011.md")),
        _ => None,
    }
}

// ── Terminal helpers ─────────────────────────────────────────────────

fn term_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
        .clamp(40, 120)
}

fn term_height() -> usize {
    terminal_size::terminal_size()
        .map(|(_, h)| h.0 as usize)
        .unwrap_or(24)
}

fn is_tty() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

// ── Inline formatting ───────────────────────────────────────────────

/// Apply inline formatting: **bold** → bold, `code` → dimmed.
///
/// Must be called AFTER textwrap so ANSI escapes don't affect width calculation.
fn format_inline(text: &str) -> String {
    let s = CODE_RE
        .replace_all(text, |caps: &regex::Captures| {
            format!("{}", caps[1].dimmed())
        })
        .into_owned();
    BOLD_RE
        .replace_all(&s, |caps: &regex::Captures| format!("{}", caps[1].bold()))
        .into_owned()
}

// ── DocRenderer ─────────────────────────────────────────────────────

struct DocRenderer {
    width: usize,
    in_code_block: bool,
    output: Vec<String>,
}

impl DocRenderer {
    fn new(width: usize) -> Self {
        Self {
            width,
            in_code_block: false,
            output: Vec::new(),
        }
    }

    fn render(&mut self, doc: &str) {
        let mut paragraph: Vec<&str> = Vec::new();

        for line in doc.lines() {
            // Code fence transitions
            if line.starts_with("```") {
                self.flush_paragraph(&mut paragraph);
                self.in_code_block = !self.in_code_block;
                self.output.push(format!("{}", line.dimmed()));
                continue;
            }

            // Inside code block: indent, dim, no wrapping
            if self.in_code_block {
                self.output.push(format!("  {}", line.dimmed()));
                continue;
            }

            // Headers
            if line.starts_with("# ") {
                self.flush_paragraph(&mut paragraph);
                self.output.push(format!("{}", line.bold().cyan()));
                continue;
            }
            if line.starts_with("## ") {
                self.flush_paragraph(&mut paragraph);
                self.output.push(format!("{}", line.bold().yellow()));
                continue;
            }
            if line.starts_with("### ") || line.starts_with("#### ") {
                self.flush_paragraph(&mut paragraph);
                self.output.push(format!("{}", line.bold()));
                continue;
            }

            // Table lines
            if line.starts_with('|') && line.ends_with('|') {
                self.flush_paragraph(&mut paragraph);
                if line.contains("---") {
                    self.output.push(format!("{}", line.dimmed()));
                } else {
                    self.output.push(format_inline(line));
                }
                continue;
            }

            // Blank line: flush paragraph
            if line.trim().is_empty() {
                self.flush_paragraph(&mut paragraph);
                self.output.push(String::new());
                continue;
            }

            // Bullet/numbered lists: wrap individually with indent
            if line.starts_with("- ") || line.starts_with("* ") || is_numbered_list(line) {
                self.flush_paragraph(&mut paragraph);
                let wrapped = textwrap::fill(line, self.width);
                // Apply inline formatting after wrapping
                self.output.push(format_inline(&wrapped));
                continue;
            }

            // Accumulate prose for paragraph wrapping
            paragraph.push(line);
        }

        self.flush_paragraph(&mut paragraph);
    }

    fn flush_paragraph(&mut self, buf: &mut Vec<&str>) {
        if buf.is_empty() {
            return;
        }
        let joined = buf.join(" ");
        let wrapped = textwrap::fill(&joined, self.width);
        // Apply inline formatting after wrapping to avoid ANSI width issues
        self.output.push(format_inline(&wrapped));
        buf.clear();
    }
}

/// Check if a line starts with a numbered list marker (e.g. "1. ", "12. ").
fn is_numbered_list(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && bytes.get(i) == Some(&b'.') && bytes.get(i + 1) == Some(&b' ')
}

// ── Pager ───────────────────────────────────────────────────────────

fn output_with_pager(lines: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = is_tty();
    let height = term_height();

    // Use pager when: TTY and content exceeds terminal height
    if is_tty && lines.len() > height {
        let pager_cmd = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
        let mut args: Vec<&str> = pager_cmd.split_whitespace().collect();
        let program = args.remove(0);

        // Add -R flag for less to pass through ANSI colors
        if program == "less" && !args.contains(&"-R") {
            args.push("-R");
        }

        if let Ok(mut child) = std::process::Command::new(program)
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                for line in lines {
                    let _ = writeln!(stdin, "{}", line);
                }
            }
            let _ = child.wait();
            return Ok(());
        }
        // Fallback if pager spawn fails
    }

    for line in lines {
        println!("{}", line);
    }
    Ok(())
}

// ── Public API ──────────────────────────────────────────────────────

/// Print per-rule documentation to stdout with terminal-aware formatting.
pub(crate) fn explain_rule(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rule = match mkdlint::rules::find_rule(name) {
        Some(r) => r,
        None => {
            eprintln!("{} unknown rule '{}'", "error:".red().bold(), name);
            suggest_similar_rules(name);
            std::process::exit(1);
        }
    };

    let canonical = rule.names()[0];

    match get_rule_doc(canonical) {
        Some(doc) => {
            let width = if is_tty() { term_width().min(100) } else { 80 };
            let mut renderer = DocRenderer::new(width);
            renderer.render(doc);
            output_with_pager(&renderer.output)
        }
        None => {
            eprintln!(
                "{} documentation not found for rule '{}'",
                "error:".red().bold(),
                canonical
            );
            std::process::exit(1);
        }
    }
}

/// Suggest rules with similar names on lookup failure.
fn suggest_similar_rules(name: &str) {
    let name_upper = name.to_uppercase();

    let mut suggestions: Vec<(&str, &str)> = Vec::new();
    for rule in mkdlint::rules::get_rules().iter() {
        let names = rule.names();
        for n in names {
            if n.to_uppercase().contains(&name_upper) || name_upper.contains(&n.to_uppercase()) {
                suggestions.push((names[0], names.get(1).copied().unwrap_or("")));
                break;
            }
        }
    }

    if !suggestions.is_empty() {
        eprintln!("\nDid you mean one of these?");
        for (id, alias) in suggestions.iter().take(5) {
            if alias.is_empty() {
                eprintln!("  {}", id);
            } else {
                eprintln!("  {} ({})", id, alias);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_have_docs() {
        for rule in mkdlint::rules::get_rules().iter() {
            let canonical = rule.names()[0];
            assert!(
                get_rule_doc(canonical).is_some(),
                "Missing documentation for rule {}",
                canonical
            );
        }
    }

    #[test]
    fn test_doc_content_not_empty() {
        for rule in mkdlint::rules::get_rules().iter() {
            let canonical = rule.names()[0];
            let doc = get_rule_doc(canonical).unwrap();
            assert!(
                !doc.is_empty(),
                "Empty documentation for rule {}",
                canonical
            );
            assert!(
                doc.contains(&format!("# {}", canonical)),
                "Documentation for {} should contain the rule name in the title",
                canonical
            );
        }
    }

    #[test]
    fn test_alias_lookup_resolves_to_doc() {
        // "heading-increment" is an alias for MD001
        let rule = mkdlint::rules::find_rule("heading-increment").unwrap();
        assert_eq!(rule.names()[0], "MD001");
        assert!(get_rule_doc("MD001").is_some());
    }

    #[test]
    fn test_unknown_rule_returns_none() {
        assert!(get_rule_doc("NONEXISTENT").is_none());
    }

    #[test]
    fn test_renderer_wraps_long_paragraph() {
        let mut r = DocRenderer::new(40);
        r.render("This is a very long paragraph that should be wrapped to fit within forty characters of terminal width.");
        // The wrapped output should have multiple lines
        let total: usize = r.output.iter().map(|l| l.lines().count()).sum();
        assert!(total > 1, "Expected wrapping, got: {:?}", r.output);
    }

    #[test]
    fn test_renderer_code_blocks_not_wrapped() {
        let mut r = DocRenderer::new(20);
        r.render(
            "```\nthis is a very long line inside a code block that should not be wrapped\n```",
        );
        // Code line should be preserved (with 2-space indent + dimmed)
        assert!(r.output[1].contains("this is a very long line"));
    }

    #[test]
    fn test_format_inline_bold_and_code() {
        // Force color output so ANSI codes are generated even when not a TTY
        colored::control::set_override(true);

        let result = format_inline("Use **--fix** to apply `auto-fix` changes.");
        assert!(result.contains("--fix"), "bold content should be present");
        assert!(
            result.contains("auto-fix"),
            "code content should be present"
        );
        // **bold** and `code` markers should be stripped, replaced with ANSI
        assert!(!result.contains("**"), "bold markers should be removed");
        assert!(result.contains('\x1b'), "Expected ANSI escape codes");

        colored::control::unset_override();
    }

    #[test]
    fn test_is_numbered_list() {
        assert!(is_numbered_list("1. First item"));
        assert!(is_numbered_list("12. Twelfth item"));
        assert!(!is_numbered_list("Not a list"));
        assert!(!is_numbered_list(". Missing number"));
    }
}
