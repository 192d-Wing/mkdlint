//! LSP backend implementation
//!
//! This module provides the main Language Server implementation.

use super::{
    code_actions, config::ConfigManager, diagnostics, document::DocumentManager, utils::Debouncer,
};
use crate::{LintOptions, apply_fixes, lint_sync};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

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

        if matching_errors.is_empty() {
            return Ok(None);
        }

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
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
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
/// `line`, `ial_col`, `cursor_col` are used to compute the replace range.
fn ial_completion_items(
    typed: &str,
    line: u32,
    ial_col: u32,
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
