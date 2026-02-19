//! Options for configuring lint operations

use crate::config::Config;
use crate::types::BoxedRule;
use std::collections::HashMap;

/// Options for linting markdown content
#[derive(Default)]
pub struct LintOptions {
    /// Files to lint (paths)
    pub files: Vec<String>,

    /// Strings to lint (keyed by identifier)
    pub strings: HashMap<String, String>,

    /// Configuration object
    pub config: Option<Config>,

    /// Path to configuration file
    pub config_file: Option<String>,

    /// Custom rules to use
    pub custom_rules: Vec<BoxedRule>,

    /// Front matter pattern (regex)
    pub front_matter: Option<String>,

    /// Whether to ignore inline configuration
    pub no_inline_config: bool,

    /// Result version for backward compatibility
    pub result_version: u32,

    /// Handle errors during rule execution
    pub handle_rule_failures: bool,

    /// Pre-built workspace heading index for cross-file MD051 validation.
    ///
    /// When provided, `lint_sync()` uses this instead of rebuilding the index
    /// from inputs. Useful for multi-pass fix convergence and watch mode.
    pub cached_workspace_headings: Option<HashMap<String, Vec<String>>>,
}

impl LintOptions {
    /// Create a new LintOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file to lint
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.files.push(file.into());
        self
    }

    /// Add multiple files to lint
    pub fn with_files(mut self, files: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Add a string to lint
    pub fn with_string(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.strings.insert(name.into(), content.into());
        self
    }

    /// Set the configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the configuration file path
    pub fn with_config_file(mut self, path: impl Into<String>) -> Self {
        self.config_file = Some(path.into());
        self
    }

    /// Add a custom rule
    pub fn with_custom_rule(mut self, rule: BoxedRule) -> Self {
        self.custom_rules.push(rule);
        self
    }

    /// Set the front matter pattern
    pub fn with_front_matter(mut self, pattern: impl Into<String>) -> Self {
        self.front_matter = Some(pattern.into());
        self
    }

    /// Disable inline configuration
    pub fn no_inline_config(mut self) -> Self {
        self.no_inline_config = true;
        self
    }
}
