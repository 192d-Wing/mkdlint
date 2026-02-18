//! Helper utilities

/// Check if a string is a valid URL
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Check if a string is empty
pub fn is_empty_string(s: &str) -> bool {
    s.is_empty()
}

/// Detect line ending style
pub fn detect_line_ending(content: &str) -> &str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Check if a trimmed line starts a code fence (``` or ~~~)
#[inline]
pub fn is_code_fence(trimmed: &str) -> bool {
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// Convert a heading text string to a GitHub-style anchor ID.
///
/// Rules: lowercase, spaces and hyphens become hyphens (de-duplicated),
/// all other non-alphanumeric characters are dropped, leading/trailing
/// hyphens are trimmed.
///
/// This matches the algorithm used by GitHub-Flavored Markdown and is
/// shared by MD051 and the LSP rename/completion handlers.
///
/// # Examples
/// ```
/// assert_eq!(mkdlint::helpers::heading_to_anchor_id("Hello World"), "hello-world");
/// assert_eq!(mkdlint::helpers::heading_to_anchor_id("What's New?"), "whats-new");
/// ```
pub fn heading_to_anchor_id(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut id = String::with_capacity(lower.len());
    let mut prev_hyphen = false;
    for ch in lower.chars() {
        if ch.is_alphanumeric() {
            id.push(ch);
            prev_hyphen = false;
        } else if (ch == ' ' || ch == '-') && !prev_hyphen {
            id.push('-');
            prev_hyphen = true;
        }
        // Skip other characters (punctuation, etc.)
    }
    id.trim_matches('-').to_string()
}

/// A heading parsed from a Markdown document, in ATX style (`# Title`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHeading {
    /// Heading level 1–6.
    pub level: usize,
    /// 0-based line index within the document.
    pub line_index: usize,
    /// Heading text, trimmed, with trailing `#` markers removed.
    pub text: String,
}

/// Parse all ATX headings from a slice of line strings, skipping code fences.
///
/// Lines may be `&str` (from `str::lines()`) or `&str` with trailing newlines
/// (from `split_inclusive`); both are handled because the code trims each line.
///
/// Returns headings in document order. Lines inside fenced code blocks
/// (``` or ~~~) are skipped.
///
/// # Examples
/// ```
/// let lines = vec!["# Title", "Some text", "## Section"];
/// let headings = mkdlint::helpers::parse_headings(&lines);
/// assert_eq!(headings.len(), 2);
/// assert_eq!(headings[0].level, 1);
/// assert_eq!(headings[0].text, "Title");
/// ```
pub fn parse_headings(lines: &[&str]) -> Vec<ParsedHeading> {
    let mut headings = Vec::new();
    let mut in_code_block = false;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_code_fence(trimmed) {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if !trimmed.starts_with('#') {
            continue;
        }
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level > 6 {
            continue;
        }
        let text = trimmed[level..].trim().trim_end_matches('#').trim();
        if text.is_empty() {
            continue;
        }
        headings.push(ParsedHeading {
            level,
            line_index: idx,
            text: text.to_string(),
        });
    }
    headings
}

/// Extract the ATX heading at a specific line (for single-line parsing).
///
/// Returns `(level, text)` or `None` if the line is not a valid heading.
///
/// # Examples
/// ```
/// assert_eq!(mkdlint::helpers::parse_heading_line("# Title"), Some((1, "Title")));
/// assert_eq!(mkdlint::helpers::parse_heading_line("## Sub ##"), Some((2, "Sub")));
/// assert_eq!(mkdlint::helpers::parse_heading_line("not a heading"), None);
/// ```
pub fn parse_heading_line(trimmed: &str) -> Option<(usize, &str)> {
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    if level > 6 {
        return None;
    }
    let text = trimmed[level..].trim().trim_end_matches('#').trim();
    if text.is_empty() {
        return None;
    }
    Some((level, text))
}

/// Collect all heading IDs from lines, handling duplicate IDs by appending `-1`, `-2`, etc.
///
/// This is used by MD051 for fragment validation and by the linting pipeline
/// for building the workspace heading index.
pub fn collect_heading_ids(lines: &[&str]) -> Vec<String> {
    let mut ids = Vec::new();
    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for heading in parse_headings(lines) {
        let base_id = heading_to_anchor_id(&heading.text);
        let count = id_counts.entry(base_id.clone()).or_insert(0);
        let final_id = if *count == 0 {
            base_id
        } else {
            format!("{}-{}", base_id, count)
        };
        *count += 1;
        ids.push(final_id);
    }

    ids
}

/// Split content into lines preserving line endings
pub fn split_lines(content: &str) -> Vec<String> {
    let line_ending = detect_line_ending(content);
    content.split(line_ending).map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com"));
        assert!(is_url("http://example.com"));
        assert!(!is_url("example.com"));
        assert!(!is_url("not a url"));
    }

    #[test]
    fn test_detect_line_ending() {
        assert_eq!(detect_line_ending("line1\nline2"), "\n");
        assert_eq!(detect_line_ending("line1\r\nline2"), "\r\n");
    }

    #[test]
    fn test_parse_headings_basic() {
        let lines = vec!["# Title", "## Section", "### Sub"];
        let h = parse_headings(&lines);
        assert_eq!(h.len(), 3);
        assert_eq!(h[0].level, 1);
        assert_eq!(h[0].line_index, 0);
        assert_eq!(h[0].text, "Title");
        assert_eq!(h[1].level, 2);
        assert_eq!(h[1].line_index, 1);
        assert_eq!(h[1].text, "Section");
    }

    #[test]
    fn test_parse_headings_skips_code_fences() {
        let lines = vec!["# Outside", "```", "# Inside fence", "```", "## After"];
        let h = parse_headings(&lines);
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].text, "Outside");
        assert_eq!(h[1].text, "After");
    }

    #[test]
    fn test_parse_headings_level_7_skipped() {
        let lines = vec!["# Valid", "####### Not a heading"];
        let h = parse_headings(&lines);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].text, "Valid");
    }

    #[test]
    fn test_parse_headings_empty_heading_skipped() {
        let lines = vec!["#", "# Real"];
        let h = parse_headings(&lines);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].text, "Real");
    }

    #[test]
    fn test_parse_heading_line() {
        assert_eq!(parse_heading_line("# Title"), Some((1, "Title")));
        assert_eq!(parse_heading_line("## Sub ## "), Some((2, "Sub")));
        assert_eq!(parse_heading_line("####### Over limit"), None);
        assert_eq!(parse_heading_line("not a heading"), None);
        assert_eq!(parse_heading_line("#"), None); // empty
    }
}
