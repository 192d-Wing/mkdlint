//! Heading extraction utilities for LSP handlers

use crate::helpers::is_code_fence;

/// A single ATX heading entry parsed from document content.
#[derive(Debug, Clone)]
pub struct HeadingEntry {
    /// Heading level 1–6
    pub level: usize,
    /// Zero-based line index
    pub line: usize,
    /// Heading text (trimmed, closing hashes stripped)
    pub text: String,
}

/// Parse all ATX headings from document content, skipping code blocks.
///
/// Returns entries in document order.
pub fn parse_headings(content: &str) -> Vec<HeadingEntry> {
    let mut headings = Vec::new();
    let mut in_code_block = false;

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if is_code_fence(trimmed) {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&level) {
                let text = trimmed[level..].trim().trim_end_matches('#').trim();
                if !text.is_empty() {
                    headings.push(HeadingEntry {
                        level,
                        line: idx,
                        text: text.to_string(),
                    });
                }
            }
        }
    }
    headings
}

/// Extract the ATX heading at a specific line index, if present.
///
/// Returns `(level, text)` or `None` if the line is not a valid heading.
pub fn heading_at_line<'a>(lines: &[&'a str], line_idx: usize) -> Option<(usize, &'a str)> {
    let raw = lines.get(line_idx)?;
    let trimmed = raw.trim();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headings_basic() {
        let content = "# Title\n## Section\n### Sub\n";
        let h = parse_headings(content);
        assert_eq!(h.len(), 3);
        assert_eq!(h[0].level, 1);
        assert_eq!(h[0].line, 0);
        assert_eq!(h[0].text, "Title");
    }

    #[test]
    fn test_parse_headings_skips_code_blocks() {
        let content = "# Outside\n```\n# Inside\n```\n## After\n";
        let h = parse_headings(content);
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].text, "Outside");
        assert_eq!(h[1].text, "After");
    }

    #[test]
    fn test_heading_at_line() {
        let lines = vec!["# Title", "text", "## Section"];
        assert_eq!(heading_at_line(&lines, 0), Some((1, "Title")));
        assert_eq!(heading_at_line(&lines, 1), None);
        assert_eq!(heading_at_line(&lines, 2), Some((2, "Section")));
    }
}
