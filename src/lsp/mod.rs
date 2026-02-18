//! Language Server Protocol (LSP) implementation for mkdlint
//!
//! This module provides a full-featured LSP server with:
//! - Real-time diagnostics on file open/edit/save
//! - Code actions (quick fixes) for fixable errors
//! - Configuration auto-discovery
//! - Debounced linting on edits
//!
//! # Example
//!
//! ```ignore
//! # use mkdlint::lsp::MkdlintLanguageServer;
//! # use tower_lsp::{LspService, Server};
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let stdin = tokio::io::stdin();
//! let stdout = tokio::io::stdout();
//!
//! let (service, socket) = LspService::new(|client| {
//!     MkdlintLanguageServer::new(client)
//! });
//!
//! Server::new(stdin, stdout, socket).serve(service).await;
//! # Ok(())
//! # }
//! ```

// Allow dead code for now since this is a work-in-progress
#![allow(dead_code)]

mod backend;
mod code_actions;
mod config;
mod diagnostics;
mod document;
mod heading;
mod utils;

pub use backend::MkdlintLanguageServer;
