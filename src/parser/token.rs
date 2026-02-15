//! Token types for parsed markdown

/// A token representing a parsed element of markdown
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Type of the token (e.g., "heading", "paragraph", "link")
    pub token_type: String,

    /// Starting line number (1-based)
    pub start_line: usize,

    /// Starting column number (1-based)
    pub start_column: usize,

    /// Ending line number (1-based)
    pub end_line: usize,

    /// Ending column number (1-based)
    pub end_column: usize,

    /// Raw text content of the token
    pub text: String,

    /// Child tokens
    pub children: Vec<usize>,

    /// Parent token index
    pub parent: Option<usize>,
}

impl Token {
    /// Create a new token
    pub fn new(token_type: impl Into<String>) -> Self {
        Self {
            token_type: token_type.into(),
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 0,
            text: String::new(),
            children: Vec::new(),
            parent: None,
        }
    }

    /// Check if this token is of a specific type
    pub fn is_type(&self, type_name: &str) -> bool {
        self.token_type == type_name
    }

    /// Check if this token matches any of the given types
    pub fn is_any_type(&self, types: &[&str]) -> bool {
        types.iter().any(|t| self.token_type == *t)
    }

    /// Get the line span of this token
    pub fn line_span(&self) -> usize {
        if self.end_line >= self.start_line {
            self.end_line - self.start_line + 1
        } else {
            0
        }
    }
}

/// Helper functions for working with token collections
pub trait TokenExt {
    /// Filter tokens by type
    fn filter_by_type(&self, token_type: &str) -> Vec<&Token>;

    /// Filter tokens by multiple types
    fn filter_by_types(&self, types: &[&str]) -> Vec<&Token>;

    /// Find parent token
    fn find_parent(&self, token: &Token) -> Option<&Token>;

    /// Get all children of a token
    fn get_children(&self, token: &Token) -> Vec<&Token>;
}

impl TokenExt for [Token] {
    fn filter_by_type(&self, token_type: &str) -> Vec<&Token> {
        self.iter().filter(|t| t.is_type(token_type)).collect()
    }

    fn filter_by_types(&self, types: &[&str]) -> Vec<&Token> {
        self.iter().filter(|t| t.is_any_type(types)).collect()
    }

    fn find_parent(&self, token: &Token) -> Option<&Token> {
        token.parent.and_then(|idx| self.get(idx))
    }

    fn get_children(&self, token: &Token) -> Vec<&Token> {
        token
            .children
            .iter()
            .filter_map(|&idx| self.get(idx))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new("heading");
        assert_eq!(token.token_type, "heading");
        assert_eq!(token.start_line, 0);
    }

    #[test]
    fn test_is_type() {
        let token = Token::new("paragraph");
        assert!(token.is_type("paragraph"));
        assert!(!token.is_type("heading"));
    }

    #[test]
    fn test_is_any_type() {
        let token = Token::new("heading");
        assert!(token.is_any_type(&["heading", "paragraph"]));
        assert!(!token.is_any_type(&["link", "image"]));
    }

    #[test]
    fn test_filter_by_type() {
        let tokens = vec![
            Token::new("heading"),
            Token::new("paragraph"),
            Token::new("heading"),
        ];

        let headings = tokens.filter_by_type("heading");
        assert_eq!(headings.len(), 2);
    }
}
