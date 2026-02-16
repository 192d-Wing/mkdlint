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
}
