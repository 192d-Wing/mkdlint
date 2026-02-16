//! LSP backend implementation
//!
//! This module provides the main Language Server implementation.

use super::{code_actions, diagnostics, document::DocumentManager, utils::Debouncer};
use crate::{LintOptions, lint_sync};
use std::sync::Arc;
use std::time::Duration;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// The mkdlint Language Server
pub struct MkdlintLanguageServer {
    client: Client,
    document_manager: Arc<DocumentManager>,
    debouncer: Arc<Debouncer>,
}

impl MkdlintLanguageServer {
    /// Create a new language server instance
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_manager: Arc::new(DocumentManager::new()),
            debouncer: Arc::new(Debouncer::new(Duration::from_millis(300))),
        }
    }

    /// Lint a document and publish diagnostics
    async fn lint_and_publish(&self, uri: Url) {
        // Get document content
        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return,
        };

        // Use URI path as file name
        let file_name = uri
            .to_file_path()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| uri.to_string());

        // Lint the document using string content
        let mut options = LintOptions::default();
        options
            .strings
            .insert(file_name.clone(), doc.content.clone());

        let results = match lint_sync(&options) {
            Ok(r) => r,
            Err(e) => {
                self.client
                    .log_message(MessageType::ERROR, format!("Lint error: {}", e))
                    .await;
                return;
            }
        };

        // Get errors for this file
        let errors = results.get(&file_name).unwrap_or(&[]).to_vec();

        // Convert errors to diagnostics
        let lines: Vec<String> = doc.content.lines().map(|s| s.to_string()).collect();
        let diagnostics: Vec<Diagnostic> = errors
            .iter()
            .map(|err| diagnostics::lint_error_to_diagnostic(err, &lines))
            .collect();

        // Update cached errors
        self.document_manager.update_errors(&uri, errors);

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for MkdlintLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "mkdlint LSP server initializing")
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "mkdlint".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "mkdlint LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "mkdlint LSP server shutting down")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        // Store document
        self.document_manager.insert(uri.clone(), content, version);

        // Lint immediately on open
        self.lint_and_publish(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get new content (full sync)
        if let Some(change) = params.content_changes.first() {
            let content = change.text.clone();

            // Update document
            self.document_manager.update(&uri, content, version);

            // Debounced lint
            let uri_clone = uri.clone();
            let self_clone = Arc::new(self.clone());
            self.debouncer.schedule(uri, async move {
                self_clone.lint_and_publish(uri_clone).await;
            });
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        // Lint immediately on save (bypass debounce)
        self.debouncer.cancel(&uri);
        self.lint_and_publish(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document
        self.document_manager.remove(&uri);

        // Cancel any pending debounced lints
        self.debouncer.cancel(&uri);

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;

        // Get document
        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get diagnostics range
        let range = params.range;

        // Find errors that overlap with the requested range
        let mut actions = Vec::new();
        for error in &doc.cached_errors {
            // Check if error has fix_info
            if error.fix_info.is_none() {
                continue;
            }

            // Check if error line is within range
            let error_line = (error.line_number - 1) as u32;
            if error_line >= range.start.line && error_line <= range.end.line {
                // Generate code action
                if let Some(action) = code_actions::fix_to_code_action(&uri, error, &doc.content) {
                    actions.push(action);
                }
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

// We need Clone for the debouncer to work
impl Clone for MkdlintLanguageServer {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            document_manager: Arc::clone(&self.document_manager),
            debouncer: Arc::clone(&self.debouncer),
        }
    }
}
