//! Error types for markdownlint

use std::fmt;

/// Main error type for markdownlint operations
#[derive(Debug, thiserror::Error)]
pub enum MarkdownlintError {
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Rule validation error
    #[error("Rule error: {0}")]
    RuleError(String),

    /// Custom rule error
    #[error("Custom rule at index {index}: {message}")]
    CustomRuleError {
        /// Index of the custom rule
        index: usize,
        /// Error message
        message: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),

    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Async runtime error
    #[cfg(feature = "async")]
    #[error("Async runtime error: {0}")]
    AsyncRuntime(String),
}

/// Result type alias for markdownlint operations
pub type Result<T> = std::result::Result<T, MarkdownlintError>;

/// Information about a lint error or warning
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintError {
    /// Line number (1-based) where the error occurs
    pub line_number: usize,

    /// Rule names (e.g., ["MD001", "heading-increment"])
    pub rule_names: Vec<String>,

    /// Rule description
    pub rule_description: String,

    /// Additional detail about the error
    pub error_detail: Option<String>,

    /// Context information (excerpt from the line)
    pub error_context: Option<String>,

    /// URL with more information about the rule
    pub rule_information: Option<String>,

    /// Column range for the error [start, length]
    pub error_range: Option<(usize, usize)>,

    /// Fix information for automatic correction
    pub fix_info: Option<FixInfo>,

    /// Severity level
    pub severity: Severity,
}

/// Severity level for lint errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Error level
    Error,
    /// Warning level
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

/// Information for automatically fixing a lint error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixInfo {
    /// Line number to apply the fix (defaults to error line if None)
    pub line_number: Option<usize>,

    /// 1-based column to start edit (None = start of line)
    pub edit_column: Option<usize>,

    /// Number of characters to delete (-1 = delete entire line)
    pub delete_count: Option<i32>,

    /// Text to insert at edit position
    pub insert_text: Option<String>,
}

impl Default for LintError {
    fn default() -> Self {
        Self {
            line_number: 0,
            rule_names: Vec::new(),
            rule_description: String::new(),
            error_detail: None,
            error_context: None,
            rule_information: None,
            error_range: None,
            fix_info: None,
            severity: Severity::Error,
        }
    }
}

impl fmt::Display for LintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} {}",
            self.line_number,
            self.severity,
            self.rule_names.join("/"),
            self.rule_description
        )?;

        if let Some(detail) = &self.error_detail {
            write!(f, " [{}]", detail)?;
        }

        if let Some(context) = &self.error_context {
            write!(f, " [Context: \"{}\"]", context)?;
        }

        Ok(())
    }
}
