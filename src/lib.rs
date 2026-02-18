//! # Markdownlint
//!
//! A Rust port of [markdownlint](https://github.com/DavidAnson/markdownlint),
//! a style checker and lint tool for Markdown/CommonMark files.
//!
//! ## Features
//!
//! - **64 built-in rules** enforcing Markdown best practices
//! - **Automatic fixing** for many rule violations
//! - **Custom rules** support via the Rule trait
//! - **Configuration** via JSON, YAML, or TOML files
//! - **Inline configuration** using HTML comments
//! - **Async and sync APIs** for flexible integration
//! - **High performance** with parallel file processing
//!
//! ## Quick Start
//!
//! ### Sync API
//!
//! ```rust,no_run
//! use mkdlint::{lint_sync, LintOptions};
//!
//! let options = LintOptions {
//!     files: vec!["README.md".to_string()],
//!     ..Default::default()
//! };
//!
//! let results = lint_sync(&options)?;
//! println!("{}", results);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Async API (requires `async` feature)
//!
//! ```rust,ignore
//! use mkdlint::{lint_async, LintOptions};
//!
//! # tokio_test::block_on(async {
//! let options = LintOptions {
//!     files: vec!["README.md".to_string()],
//!     ..Default::default()
//! };
//!
//! let results = lint_async(&options).await?;
//! println!("{}", results);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # })
//! ```
//!
//! ## Configuration
//!
//! Configuration can be provided via files or directly in options:
//!
//! ```json
//! {
//!   "default": true,
//!   "MD013": false,
//!   "MD033": {
//!     "allowed_elements": ["br", "img"]
//!   }
//! }
//! ```
//!
//! ## Inline Configuration
//!
//! Rules can be disabled/enabled using HTML comments:
//!
//! ```markdown
//! <!-- markdownlint-disable MD013 -->
//! This line can be as long as you want.
//! <!-- markdownlint-enable MD013 -->
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod formatters;
pub mod helpers;
pub mod lint;
pub mod parser;
pub mod rules;
pub mod types;

#[cfg(feature = "lsp")]
pub mod lsp;

// Re-export main types and functions
pub use config::{Config, ConfigParser, RuleConfig};
pub use lint::{apply_fixes, lint_sync};
pub use types::{LintError, LintOptions, LintResults, Rule, RuleParams};

#[cfg(feature = "async")]
pub use lint::lint_async;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the library version
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
