//! LSP backend implementation
//!
//! This module provides the main Language Server implementation.

use super::{
    code_actions, config::ConfigManager, diagnostics, document::DocumentManager, utils::Debouncer,
};
use crate::{LintOptions, apply_fixes, lint_sync};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Regex that captures the fragment portion in a markdown anchor link `(#fragment)`.
/// Matches `(#` followed by the fragment up to `)`, `"`, `'`, or whitespace.
static ANCHOR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\(#([^)"'\s]+)"#).expect("valid regex"));

/// Walk a directory recursively and collect `.md`/`.markdown` files.
///
/// Skips hidden directories (starting with `.`) and common build directories
/// (`node_modules`, `target`, `vendor`) to avoid scanning irrelevant paths.
fn walkdir_md(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_recursive(root, &mut files);
    Ok(files)
}

fn walk_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // Skip hidden dirs and common build dirs
            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "vendor"
            {
                continue;
            }
        }
        if path.is_dir() {
            walk_recursive(&path, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str())
            && (ext == "md" || ext == "markdown")
        {
            out.push(path);
        }
    }
}

/// The mkdlint Language Server
pub struct MkdlintLanguageServer {
    client: Client,
    document_manager: Arc<DocumentManager>,
    config_manager: Arc<Mutex<ConfigManager>>,
    debouncer: Arc<Debouncer>,
}

impl MkdlintLanguageServer {
    /// Create a new language server instance
    pub fn new(client: Client) -> Self {
        // Start with empty workspace roots, will be set in initialize()
        Self {
            client,
            document_manager: Arc::new(DocumentManager::new()),
            config_manager: Arc::new(Mutex::new(ConfigManager::new(vec![]))),
            debouncer: Arc::new(Debouncer::new(Duration::from_millis(300))),
        }
    }

    /// Scan workspace roots for `.md` files and publish diagnostics for each.
    ///
    /// Called once after initialization to populate the Problems panel with
    /// errors from files the user hasn't opened yet.
    async fn scan_workspace(&self) {
        let roots: Vec<PathBuf> = self.config_manager.lock().unwrap().workspace_roots.clone();

        if roots.is_empty() {
            return;
        }

        let mut md_files: Vec<PathBuf> = Vec::new();
        for root in &roots {
            if let Ok(entries) = walkdir_md(root) {
                md_files.extend(entries);
            }
        }

        if md_files.is_empty() {
            return;
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!("Scanning {} markdown file(s) in workspace", md_files.len()),
            )
            .await;

        for path in md_files {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let file_name = path.to_string_lossy().to_string();
            let uri = match Url::from_file_path(&path) {
                Ok(u) => u,
                Err(_) => continue,
            };

            // Skip files already open (they have fresher diagnostics)
            if self.document_manager.get(&uri).is_some() {
                continue;
            }

            let config = self.config_manager.lock().unwrap().discover_config(&uri);

            let mut options = LintOptions::default();
            options.strings.insert(file_name.clone(), content.clone());
            if let Some(config) = config {
                options.config = Some(config);
            }

            let results = match lint_sync(&options) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let errors = results.get(&file_name).unwrap_or(&[]).to_vec();
            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            let diags: Vec<Diagnostic> = errors
                .iter()
                .filter(|err| !err.fix_only)
                .map(|err| diagnostics::lint_error_to_diagnostic(err, &lines))
                .collect();

            if !diags.is_empty() {
                self.client.publish_diagnostics(uri, diags, None).await;
            }
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

        // Discover config for this file
        let config = self.config_manager.lock().unwrap().discover_config(&uri);

        // Lint the document using string content
        let mut options = LintOptions::default();
        options
            .strings
            .insert(file_name.clone(), doc.content.clone());

        // Apply config if found
        if let Some(config) = config {
            options.config = Some(config);
        }

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
            .filter(|err| !err.fix_only)
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
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "mkdlint LSP server initializing")
            .await;

        // Extract workspace roots from initialize params
        let workspace_roots: Vec<PathBuf> = params
            .workspace_folders
            .unwrap_or_default()
            .into_iter()
            .filter_map(|folder| folder.uri.to_file_path().ok())
            .collect();

        // If no workspace folders, try root_uri
        let workspace_roots = if workspace_roots.is_empty() {
            params
                .root_uri
                .and_then(|uri| uri.to_file_path().ok())
                .map(|path| vec![path])
                .unwrap_or_default()
        } else {
            workspace_roots
        };

        // Extract preset from initialization options (e.g. from VS Code setting `mkdlint.preset`)
        let preset_override: Option<String> = params
            .initialization_options
            .as_ref()
            .and_then(|o| o.get("preset"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Update config manager with workspace roots and optional preset override
        *self.config_manager.lock().unwrap() =
            ConfigManager::with_preset(workspace_roots, preset_override);

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "mkdlint LSP initialized with {} workspace root(s)",
                    self.config_manager.lock().unwrap().workspace_roots.len()
                ),
            )
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["mkdlint.fixAll".to_string()],
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "{".to_string(),
                        " ".to_string(),
                        ".".to_string(),
                        "#".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                // Declare that we handle workspace/didChangeConfiguration
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: None,
                    file_operations: None,
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "mkdlint".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        // Register for config file change notifications
        let watchers = vec![
            FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/.markdownlint.json".to_string()),
                kind: Some(WatchKind::all()),
            },
            FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/.markdownlint.jsonc".to_string()),
                kind: Some(WatchKind::all()),
            },
            FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/.markdownlint.yaml".to_string()),
                kind: Some(WatchKind::all()),
            },
            FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/.markdownlint.yml".to_string()),
                kind: Some(WatchKind::all()),
            },
            FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/.markdownlintrc".to_string()),
                kind: Some(WatchKind::all()),
            },
        ];

        let registration = Registration {
            id: "config-watcher".to_string(),
            method: "workspace/didChangeWatchedFiles".to_string(),
            register_options: Some(
                serde_json::to_value(DidChangeWatchedFilesRegistrationOptions { watchers })
                    .unwrap(),
            ),
        };

        // Also register for workspace/didChangeConfiguration
        let config_registration = Registration {
            id: "config-change-watcher".to_string(),
            method: "workspace/didChangeConfiguration".to_string(),
            register_options: None,
        };

        if let Err(e) = self
            .client
            .register_capability(vec![registration, config_registration])
            .await
        {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("Failed to register file watchers: {}", e),
                )
                .await;
        }

        self.client
            .log_message(MessageType::INFO, "mkdlint LSP server initialized")
            .await;

        // Scan workspace for .md files and publish initial diagnostics
        self.scan_workspace().await;
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

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        // Config file changed — invalidate cache and re-lint all open documents
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Config file change detected ({} file(s)), re-linting open documents",
                    params.changes.len()
                ),
            )
            .await;

        self.config_manager.lock().unwrap().clear_cache();

        // Re-lint all open documents
        let uris = self.document_manager.all_uris();
        for uri in uris {
            self.lint_and_publish(uri).await;
        }
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        // Fetch the current mkdlint.preset value from the client
        let config_items = vec![ConfigurationItem {
            scope_uri: None,
            section: Some("mkdlint.preset".to_string()),
        }];

        let new_preset: Option<String> = match self.client.configuration(config_items).await {
            Ok(values) => values
                .into_iter()
                .next()
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Failed to fetch mkdlint.preset config: {e}"),
                    )
                    .await;
                return;
            }
        };

        // Update the preset override and clear cache so next lint picks it up
        {
            let mut mgr = self.config_manager.lock().unwrap();
            mgr.preset_override = new_preset.clone();
            mgr.clear_cache();
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "mkdlint.preset changed to {:?}, re-linting open documents",
                    new_preset
                ),
            )
            .await;

        // Re-lint all open documents with the new preset
        let uris = self.document_manager.all_uris();
        for uri in uris {
            self.lint_and_publish(uri).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Find errors at the hover position
        let hover_line = position.line as usize + 1; // Convert 0-based to 1-based
        let matching_errors: Vec<_> = doc
            .cached_errors
            .iter()
            .filter(|e| e.line_number == hover_line)
            .collect();

        let mut sections = Vec::new();
        for error in &matching_errors {
            let rule_id = error.rule_names.first().unwrap_or(&"unknown");
            let rule_alias = error.rule_names.get(1).unwrap_or(rule_id);

            let mut md = format!("### {} / {}\n\n", rule_id, rule_alias);
            md.push_str(error.rule_description);
            md.push('\n');

            if let Some(detail) = &error.error_detail {
                md.push_str(&format!("\n**Detail:** {}\n", detail));
            }

            if let Some(suggestion) = &error.suggestion {
                md.push_str(&format!("\n**Suggestion:** {}\n", suggestion));
            }

            if error.fix_info.is_some() {
                md.push_str("\n*Auto-fixable* \u{1f527}\n");
            }

            if let Some(url) = error.rule_information {
                md.push_str(&format!("\n[Documentation]({})\n", url));
            }

            sections.push(md);
        }

        // If hovering over a rule name/alias (e.g. in a disable comment), show rule docs
        if let Some(line_text) = doc.content.lines().nth(position.line as usize) {
            let col = position.character as usize;
            if let Some(word) = extract_word(line_text, col) {
                // Check if the word matches any rule name or alias
                let rules = crate::rules::get_rules();
                if let Some(rule) = rules
                    .iter()
                    .find(|r| r.names().iter().any(|n| n.eq_ignore_ascii_case(word)))
                {
                    // Only show rule doc hover if it's not already shown via an error
                    let already_shown = matching_errors
                        .iter()
                        .any(|e| e.rule_names.iter().any(|n| n.eq_ignore_ascii_case(word)));
                    if !already_shown {
                        let names = rule.names();
                        let rule_id = names.first().copied().unwrap_or("unknown");
                        let rule_alias = names.get(1).copied().unwrap_or(rule_id);
                        let mut md = format!("### {} / {}\n\n", rule_id, rule_alias);
                        md.push_str(rule.description());
                        md.push('\n');
                        sections.push(md);
                    }
                }
            }
        }

        if sections.is_empty() {
            return Ok(None);
        }

        let contents = sections.join("\n---\n\n");

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: contents,
            }),
            range: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let lines: Vec<&str> = doc.content.lines().collect();
        let line = match lines.get(position.line as usize) {
            Some(l) => *l,
            None => return Ok(None),
        };

        // Only offer completions when the cursor is within or just after `{:`
        // Look backwards from the cursor to find the start of an IAL
        let col = position.character as usize;
        let prefix = &line[..col.min(line.len())];

        // ── Link anchor completion: [text](#   or   [text](#partial ──────────
        // Detect if the cursor is inside a link's fragment: `[...](#`
        if let Some(anchor_start) = prefix.rfind("(#") {
            // Make sure there's no `)` closing the link between `(#` and cursor
            if !prefix[anchor_start..].contains(')') {
                // The partial anchor text the user has typed after `(#`
                let typed_anchor = &prefix[anchor_start + 2..];

                // Collect heading anchors from the document
                let lines: Vec<&str> = doc.content.lines().collect();
                let mut in_code_block = false;
                let mut items: Vec<CompletionItem> = Vec::new();

                for (idx, l) in lines.iter().enumerate() {
                    let trimmed = l.trim();
                    if crate::helpers::is_code_fence(trimmed) {
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
                            if text.is_empty() {
                                continue;
                            }
                            let anchor = crate::helpers::heading_to_anchor_id(text);
                            if !anchor.starts_with(typed_anchor) {
                                continue;
                            }
                            // Replace range: from just after `(#` to cursor
                            let replace_start = (anchor_start as u32 + 2).min(col as u32);
                            let replace_range = Range {
                                start: Position {
                                    line: position.line,
                                    character: replace_start,
                                },
                                end: Position {
                                    line: position.line,
                                    character: col as u32,
                                },
                            };
                            items.push(CompletionItem {
                                label: anchor.clone(),
                                kind: Some(CompletionItemKind::REFERENCE),
                                detail: Some(format!("Line {}: {}", idx + 1, text)),
                                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                                    range: replace_range,
                                    new_text: anchor,
                                })),
                                ..Default::default()
                            });
                        }
                    }
                }

                return Ok(Some(CompletionResponse::Array(items)));
            }
        }

        // ── Cross-file link completion: [text](./ or [text]( ────────────────
        // Detect if the cursor is inside a file link href (not starting with `#`)
        if let Some(href_start) = prefix.rfind("](")
            && !prefix[href_start..].contains(')')
        {
            let typed_path = &prefix[href_start + 2..]; // text after `](`
            // Only handle file links (not anchors — those are handled above)
            if !typed_path.starts_with('#') {
                let doc_uri = uri.clone();
                let doc_dir = doc_uri
                    .to_file_path()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()));

                let roots = self.config_manager.lock().unwrap().workspace_roots.clone();
                let mut items: Vec<CompletionItem> = Vec::new();

                for root in &roots {
                    if let Ok(files) = walkdir_md(root) {
                        for file_path in files {
                            // Compute a relative path from the document's directory
                            let rel = if let Some(ref dir) = doc_dir {
                                file_path
                                    .strip_prefix(dir)
                                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                                    .or_else(|_| {
                                        // Cross-directory: try relative from workspace root
                                        file_path.strip_prefix(root).map(|p| {
                                            format!("/{}", p.to_string_lossy().replace('\\', "/"))
                                        })
                                    })
                                    .unwrap_or_else(|_| file_path.to_string_lossy().into_owned())
                            } else {
                                file_path.to_string_lossy().into_owned()
                            };

                            if rel.starts_with(typed_path) {
                                let replace_start = (href_start as u32 + 2).min(col as u32);
                                let replace_range = Range {
                                    start: Position {
                                        line: position.line,
                                        character: replace_start,
                                    },
                                    end: Position {
                                        line: position.line,
                                        character: col as u32,
                                    },
                                };
                                items.push(CompletionItem {
                                    label: rel.clone(),
                                    kind: Some(CompletionItemKind::FILE),
                                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                                        range: replace_range,
                                        new_text: rel,
                                    })),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }

                if !items.is_empty() {
                    return Ok(Some(CompletionResponse::Array(items)));
                }
            }
        }

        // ── Kramdown IAL completion: {: ... } ────────────────────────────────
        // Find last `{:` before the cursor (within the same line)
        let ial_start = match prefix.rfind("{:") {
            Some(pos) => pos,
            None => return Ok(None),
        };

        // Ensure no closing `}` between `{:` and cursor (i.e. we are inside the IAL)
        if prefix[ial_start..].contains('}') {
            return Ok(None);
        }

        // The text the user has typed since `{: ` — used for filtering
        let typed = prefix[ial_start + 2..].trim_start();

        let items = ial_completion_items(typed, position.line, ial_start as u32, col as u32);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let lines: Vec<&str> = doc.content.lines().collect();
        let total_lines = lines.len() as u32;

        // Parse headings from document content
        let mut headings: Vec<(usize, u32, String)> = Vec::new(); // (level, line, text)
        let mut in_code_block = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if crate::helpers::is_code_fence(trimmed) {
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
                        headings.push((level, idx as u32, text.to_string()));
                    }
                }
            }
        }

        if headings.is_empty() {
            return Ok(Some(DocumentSymbolResponse::Nested(vec![])));
        }

        // Build nested DocumentSymbol tree using a stack-based approach
        fn build_tree(headings: &[(usize, u32, String)], total_lines: u32) -> Vec<DocumentSymbol> {
            if headings.is_empty() {
                return vec![];
            }

            // For each heading, compute end line (just before the next heading at same or higher level, or EOF)
            let end_lines: Vec<u32> = headings
                .iter()
                .enumerate()
                .map(|(i, (level, _, _))| {
                    // Find next heading at same or higher (lower number) level
                    for h in &headings[(i + 1)..] {
                        if h.0 <= *level {
                            return h.1.saturating_sub(1);
                        }
                    }
                    total_lines.saturating_sub(1)
                })
                .collect();

            // Recursive: build symbols for headings at the current nesting level
            fn build_level(
                headings: &[(usize, u32, String)],
                end_lines: &[u32],
                start: usize,
                end: usize,
                parent_level: usize,
            ) -> Vec<DocumentSymbol> {
                let mut symbols = Vec::new();
                let mut i = start;
                while i < end {
                    let (level, line, ref text) = headings[i];
                    if level != parent_level {
                        i += 1;
                        continue;
                    }

                    // Find children: headings between this one and the next sibling
                    let sibling_end = {
                        let mut j = i + 1;
                        while j < end && headings[j].0 > level {
                            j += 1;
                        }
                        j
                    };

                    let children = if sibling_end > i + 1 {
                        // Find the min child level
                        let child_level = headings[i + 1..sibling_end]
                            .iter()
                            .map(|(l, _, _)| *l)
                            .min()
                            .unwrap_or(level + 1);
                        build_level(headings, end_lines, i + 1, sibling_end, child_level)
                    } else {
                        vec![]
                    };

                    let end_line = end_lines[i];
                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: text.clone(),
                        detail: Some(format!("h{}", level)),
                        kind: SymbolKind::STRING,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line, character: 0 },
                            end: Position {
                                line: end_line,
                                character: 0,
                            },
                        },
                        selection_range: Range {
                            start: Position { line, character: 0 },
                            end: Position {
                                line,
                                character: text.len() as u32 + level as u32 + 1,
                            },
                        },
                        children: if children.is_empty() {
                            None
                        } else {
                            Some(children)
                        },
                    });
                    i = sibling_end;
                }
                symbols
            }

            let top_level = headings.iter().map(|(l, _, _)| *l).min().unwrap_or(1);
            build_level(headings, &end_lines, 0, headings.len(), top_level)
        }

        let symbols = build_tree(&headings, total_lines);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Only format if there are fixable errors
        let has_fixes = doc.cached_errors.iter().any(|e| e.fix_info.is_some());
        if !has_fixes {
            return Ok(None);
        }

        let fixed_content = apply_fixes(&doc.content, &doc.cached_errors);
        if fixed_content == doc.content {
            return Ok(None);
        }

        // Replace entire document content
        let line_count = doc.content.lines().count() as u32;
        let last_line_len = doc.content.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

        let text_edit = TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: line_count,
                    character: last_line_len,
                },
            },
            new_text: fixed_content,
        };

        Ok(Some(vec![text_edit]))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let lines: Vec<&str> = doc.content.lines().collect();
        let mut ranges = Vec::new();
        let mut in_code_block = false;
        let mut code_block_start: Option<u32> = None;

        // Track headings for section folding
        let mut heading_stack: Vec<(usize, u32)> = Vec::new(); // (level, start_line)

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_num = idx as u32;

            // Code block folding
            if crate::helpers::is_code_fence(trimmed) {
                if in_code_block {
                    // End of code block
                    if let Some(start) = code_block_start.take()
                        && line_num > start
                    {
                        ranges.push(FoldingRange {
                            start_line: start,
                            start_character: None,
                            end_line: line_num,
                            end_character: None,
                            kind: Some(FoldingRangeKind::Region),
                            collapsed_text: None,
                        });
                    }
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    code_block_start = Some(line_num);
                }
                continue;
            }

            if in_code_block {
                continue;
            }

            // Heading section folding
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                if (1..=6).contains(&level) {
                    // Close all headings at same or deeper level
                    while let Some(&(prev_level, prev_start)) = heading_stack.last() {
                        if prev_level >= level {
                            heading_stack.pop();
                            let end = line_num.saturating_sub(1);
                            if end > prev_start {
                                ranges.push(FoldingRange {
                                    start_line: prev_start,
                                    start_character: None,
                                    end_line: end,
                                    end_character: None,
                                    kind: Some(FoldingRangeKind::Region),
                                    collapsed_text: None,
                                });
                            }
                        } else {
                            break;
                        }
                    }
                    heading_stack.push((level, line_num));
                }
            }
        }

        // Close remaining headings at EOF
        let last_line = lines.len().saturating_sub(1) as u32;
        for (_, start) in heading_stack {
            if last_line > start {
                ranges.push(FoldingRange {
                    start_line: start,
                    start_character: None,
                    end_line: last_line,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
        }

        if ranges.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ranges))
        }
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let line_idx = params.position.line as usize;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let lines: Vec<&str> = doc.content.lines().collect();
        let line = match lines.get(line_idx) {
            Some(l) => l.trim(),
            None => return Ok(None),
        };

        // Only allow rename on ATX heading lines
        if !line.starts_with('#') {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Rename is only supported on heading lines",
            ));
        }
        let level = line.chars().take_while(|&c| c == '#').count();
        if level > 6 {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Not a valid heading",
            ));
        }

        // Compute the range of heading text (after `## `)
        let raw_line = lines[line_idx]; // original (unstripped) line
        let hashes_and_space = level + 1; // `## ` = level chars + 1 space
        let text_start = hashes_and_space.min(raw_line.len());
        let text = raw_line[text_start..].trim_end_matches('#').trim();
        if text.is_empty() {
            return Err(tower_lsp::jsonrpc::Error::invalid_params("Empty heading"));
        }

        // Find the character offset of the text in the original line
        let char_start = raw_line.find(text).unwrap_or(text_start) as u32;
        let char_end = char_start + text.len() as u32;

        Ok(Some(PrepareRenameResponse::Range(Range {
            start: Position {
                line: params.position.line,
                character: char_start,
            },
            end: Position {
                line: params.position.line,
                character: char_end,
            },
        })))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let line_idx = params.text_document_position.position.line as usize;
        let new_name = &params.new_name;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let lines: Vec<&str> = doc.content.lines().collect();
        let raw_line = match lines.get(line_idx) {
            Some(l) => *l,
            None => return Ok(None),
        };
        let trimmed = raw_line.trim();

        // Extract old heading text
        if !trimmed.starts_with('#') {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Position is not a heading",
            ));
        }
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level > 6 {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Not a valid heading",
            ));
        }
        let old_text = trimmed[level..].trim().trim_end_matches('#').trim();
        let old_slug = crate::helpers::heading_to_anchor_id(old_text);
        let new_slug = crate::helpers::heading_to_anchor_id(new_name);

        // Build hashes prefix (e.g. "## ")
        let hashes: String = "#".repeat(level);
        let new_heading_line = format!("{} {}", hashes, new_name);

        let mut edits: Vec<TextEdit> = Vec::new();

        // 1. Replace the heading line itself
        edits.push(TextEdit {
            range: Range {
                start: Position {
                    line: line_idx as u32,
                    character: 0,
                },
                end: Position {
                    line: line_idx as u32,
                    character: raw_line.len() as u32,
                },
            },
            new_text: new_heading_line,
        });

        // 2. Update same-document anchor links `[label](#old-slug)` and
        //    `[label](#old-slug "title")` — replace only the fragment part.
        // (ANCHOR_RE is declared at module level)
        for (idx, l) in lines.iter().enumerate() {
            if idx == line_idx {
                continue; // skip the heading line we already handled
            }
            for cap in ANCHOR_RE.captures_iter(l) {
                let fragment = &cap[1];
                if fragment == old_slug {
                    // Find byte offset of this match in the line
                    let match_start = cap.get(1).unwrap().start() as u32;
                    let match_end = cap.get(1).unwrap().end() as u32;
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: idx as u32,
                                character: match_start,
                            },
                            end: Position {
                                line: idx as u32,
                                character: match_end,
                            },
                        },
                        new_text: new_slug.clone(),
                    });
                }
            }
        }

        let mut changes = HashMap::new();
        changes.insert(uri, edits);
        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let line_idx = params.text_document_position.position.line as usize;
        let col = params.text_document_position.position.character as usize;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let lines: Vec<&str> = doc.content.lines().collect();
        let raw_line = match lines.get(line_idx) {
            Some(l) => *l,
            None => return Ok(None),
        };
        let trimmed = raw_line.trim();

        // Determine the target anchor slug from the cursor position:
        //   1. Cursor on a heading → use that heading's slug
        //   2. Cursor inside (#anchor) → use that anchor
        //   3. Otherwise → no references
        let target_slug: String;

        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= 6 {
                let text = trimmed[level..].trim().trim_end_matches('#').trim();
                if text.is_empty() {
                    return Ok(None);
                }
                target_slug = crate::helpers::heading_to_anchor_id(text);
            } else {
                return Ok(None);
            }
        } else {
            // Try to find an anchor link under the cursor
            let mut found = None;
            for cap in ANCHOR_RE.captures_iter(raw_line) {
                let frag_match = cap.get(1).unwrap();
                // The `(#` starts one char before the captured group
                let anchor_start = frag_match.start().saturating_sub(1);
                let anchor_end = frag_match.end();
                if col >= anchor_start && col <= anchor_end {
                    found = Some(frag_match.as_str().to_string());
                    break;
                }
            }
            match found {
                Some(slug) => target_slug = slug,
                None => return Ok(None),
            }
        }

        // Scan all lines for (#target_slug) references
        let mut locations: Vec<Location> = Vec::new();
        for (idx, l) in lines.iter().enumerate() {
            for cap in ANCHOR_RE.captures_iter(l) {
                if cap[1] == *target_slug {
                    let frag_match = cap.get(1).unwrap();
                    // Range covers the full `(#slug)` — start at `(`
                    let char_start = (frag_match.start() as u32).saturating_sub(1);
                    let char_end = frag_match.end() as u32 + 1; // past `)`
                    locations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: idx as u32,
                                character: char_start,
                            },
                            end: Position {
                                line: idx as u32,
                                character: char_end,
                            },
                        },
                    });
                }
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let line_idx = params.text_document_position_params.position.line as usize;
        let col = params.text_document_position_params.position.character as usize;

        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let lines: Vec<&str> = doc.content.lines().collect();
        let raw_line = match lines.get(line_idx) {
            Some(l) => *l,
            None => return Ok(None),
        };

        // Find the anchor slug the cursor is hovering over in `(#slug)`
        let mut target_slug: Option<String> = None;
        for cap in ANCHOR_RE.captures_iter(raw_line) {
            let frag_match = cap.get(1).unwrap();
            let anchor_start = frag_match.start().saturating_sub(1); // includes `#`
            let anchor_end = frag_match.end();
            if col >= anchor_start && col <= anchor_end {
                target_slug = Some(cap[1].to_string());
                break;
            }
        }

        let slug = match target_slug {
            Some(s) => s,
            None => return Ok(None),
        };

        // Find the heading whose slug matches
        let mut in_code_block = false;
        for (idx, l) in lines.iter().enumerate() {
            let trimmed = l.trim();
            if crate::helpers::is_code_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                if level > 6 {
                    continue;
                }
                let text = trimmed[level..].trim().trim_end_matches('#').trim();
                if text.is_empty() {
                    continue;
                }
                if crate::helpers::heading_to_anchor_id(text) == slug {
                    let heading_end = l.len() as u32;
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri,
                        range: Range {
                            start: Position {
                                line: idx as u32,
                                character: 0,
                            },
                            end: Position {
                                line: idx as u32,
                                character: heading_end,
                            },
                        },
                    })));
                }
            }
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;

        // Get document
        let doc = match self.document_manager.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get diagnostics range and context diagnostics
        let range = params.range;
        let context_diagnostics = params.context.diagnostics;

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
                // Match this error to a context diagnostic by line and rule code
                let matched_diag = context_diagnostics.iter().find(|d| {
                    d.range.start.line == error_line
                        && error.rule_names.first().is_some_and(|name| {
                            d.code == Some(NumberOrString::String(name.to_string()))
                        })
                });

                // Generate code action, linking to the matched diagnostic
                if let Some(action) = code_actions::fix_to_code_action(
                    &uri,
                    error,
                    &doc.content,
                    matched_diag.cloned(),
                ) {
                    actions.push(action);
                }
            }
        }

        // Add "Fix All" command if there are any fixable errors in the document
        let fixable_count = doc
            .cached_errors
            .iter()
            .filter(|e| e.fix_info.is_some())
            .count();
        if fixable_count > 0 {
            let fix_all_command = CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Fix all mkdlint issues ({} fixes)", fixable_count),
                kind: Some(CodeActionKind::SOURCE_FIX_ALL),
                command: Some(Command {
                    title: "Fix all".to_string(),
                    command: "mkdlint.fixAll".to_string(),
                    arguments: Some(vec![serde_json::to_value(&uri).unwrap()]),
                }),
                ..Default::default()
            });
            actions.push(fix_all_command);
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "mkdlint.fixAll" => {
                // Extract URI from arguments
                let uri = match params.arguments.first() {
                    Some(arg) => match serde_json::from_value::<Url>(arg.clone()) {
                        Ok(uri) => uri,
                        Err(e) => {
                            self.client
                                .log_message(
                                    MessageType::ERROR,
                                    format!("Invalid URI argument: {}", e),
                                )
                                .await;
                            return Ok(None);
                        }
                    },
                    None => {
                        self.client
                            .log_message(MessageType::ERROR, "No URI provided for fixAll")
                            .await;
                        return Ok(None);
                    }
                };

                // Get document
                let doc = match self.document_manager.get(&uri) {
                    Some(doc) => doc,
                    None => {
                        self.client
                            .log_message(MessageType::ERROR, format!("Document not found: {}", uri))
                            .await;
                        return Ok(None);
                    }
                };

                // Apply all fixes
                let fixed_content = apply_fixes(&doc.content, &doc.cached_errors);

                // Create workspace edit to replace entire document
                let text_edit = TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: u32::MAX,
                            character: u32::MAX,
                        },
                    },
                    new_text: fixed_content.clone(),
                };

                let mut changes = HashMap::new();
                changes.insert(uri.clone(), vec![text_edit]);

                let workspace_edit = WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                };

                // Apply the edit
                if let Ok(response) = self.client.apply_edit(workspace_edit).await {
                    if response.applied {
                        self.client
                            .log_message(MessageType::INFO, "Applied all fixes")
                            .await;

                        // Update document content
                        self.document_manager
                            .update(&uri, fixed_content, doc.version + 1);

                        // Re-lint the document
                        self.lint_and_publish(uri).await;
                    } else {
                        self.client
                            .log_message(
                                MessageType::ERROR,
                                format!(
                                    "Failed to apply fixes: {}",
                                    response.failure_reason.unwrap_or_default()
                                ),
                            )
                            .await;
                    }
                }

                Ok(None)
            }
            _ => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Unknown command: {}", params.command),
                    )
                    .await;
                Ok(None)
            }
        }
    }
}

/// Build completion items for Kramdown IAL syntax `{: ...}`.
///
/// Offers:
/// - `#id`            — ID selector
/// - `.class`         — class selector
/// - Common HTML/aria attributes with `=` snippets
///
/// `typed` is what the user has typed after `{: ` (used to filter).
/// `line` and `cursor_col` are used to compute the replace range.
fn ial_completion_items(
    typed: &str,
    line: u32,
    _ial_col: u32,
    cursor_col: u32,
) -> Vec<CompletionItem> {
    // Replace range: from the start of what the user typed (after `{:` + whitespace) to the cursor.
    // `typed` has leading whitespace stripped, so its length tells us how far back to start.
    let replace_start = cursor_col.saturating_sub(typed.len() as u32);
    let replace_range = Range {
        start: Position {
            line,
            character: replace_start,
        },
        end: Position {
            line,
            character: cursor_col,
        },
    };

    let mut items: Vec<CompletionItem> = Vec::new();

    // Helper to create an item with optional snippet insert text
    let make_item = |label: &'static str,
                     kind: CompletionItemKind,
                     detail: &'static str,
                     insert: &'static str,
                     is_snippet: bool| {
        CompletionItem {
            label: label.to_string(),
            kind: Some(kind),
            detail: Some(detail.to_string()),
            insert_text: Some(insert.to_string()),
            insert_text_format: Some(if is_snippet {
                InsertTextFormat::SNIPPET
            } else {
                InsertTextFormat::PLAIN_TEXT
            }),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range: replace_range,
                new_text: insert.to_string(),
            })),
            ..Default::default()
        }
    };

    // ID selector
    items.push(make_item(
        "#",
        CompletionItemKind::VALUE,
        "ID attribute",
        "#${1:id}",
        true,
    ));

    // Class selector
    items.push(make_item(
        ".",
        CompletionItemKind::VALUE,
        "CSS class",
        ".${1:class}",
        true,
    ));

    // Common HTML attributes as key=value pairs
    let attrs: &[(&str, &str, &str)] = &[
        ("id", "id=\"…\"", "id=\"${1:value}\""),
        ("class", "class=\"…\"", "class=\"${1:value}\""),
        ("lang", "lang=\"…\"", "lang=\"${1:en}\""),
        ("dir", "dir=\"…\"", "dir=\"${1:ltr}\""),
        ("style", "style=\"…\"", "style=\"${1:property: value}\""),
        ("tabindex", "tabindex=\"…\"", "tabindex=\"${1:0}\""),
        ("role", "role=\"…\"", "role=\"${1:region}\""),
        (
            "aria-label",
            "aria-label=\"…\"",
            "aria-label=\"${1:label}\"",
        ),
        (
            "aria-describedby",
            "aria-describedby=\"…\"",
            "aria-describedby=\"${1:id}\"",
        ),
        (
            "aria-hidden",
            "aria-hidden=\"…\"",
            "aria-hidden=\"${1:true}\"",
        ),
        ("data-", "data-*=\"…\"", "data-${1:key}=\"${2:value}\""),
    ];

    for (label, detail, snippet) in attrs {
        items.push(make_item(
            label,
            CompletionItemKind::PROPERTY,
            detail,
            snippet,
            true,
        ));
    }

    // Filter: keep items whose label starts with what was typed,
    // OR single-char selector items (#, .) when the user started with that char.
    if !typed.is_empty() {
        items.retain(|item| {
            item.label.starts_with(typed)
                || (item.label.len() == 1 && typed.starts_with(item.label.as_str()))
        });
    }

    items
}

/// Extract the word (alphanumeric + `-`) under `col` in `line`.
/// Returns `None` if the character at `col` is not a word character.
fn extract_word(line: &str, col: usize) -> Option<&str> {
    let chars: Vec<char> = line.chars().collect();
    if col >= chars.len() {
        return None;
    }
    let is_word_char = |c: char| c.is_alphanumeric() || c == '-';
    if !is_word_char(chars[col]) {
        return None;
    }
    // Find start
    let mut start = col;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    // Find end
    let mut end = col + 1;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }
    // Convert char indices to byte indices
    let byte_start: usize = chars[..start].iter().map(|c| c.len_utf8()).sum();
    let byte_end: usize = chars[..end].iter().map(|c| c.len_utf8()).sum();
    Some(&line[byte_start..byte_end])
}

// We need Clone for the debouncer to work
impl Clone for MkdlintLanguageServer {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            document_manager: Arc::clone(&self.document_manager),
            config_manager: Arc::clone(&self.config_manager),
            debouncer: Arc::clone(&self.debouncer),
        }
    }
}
