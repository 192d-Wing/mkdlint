#![cfg(feature = "lsp")]

//! Integration tests for mkdlint LSP server

use mkdlint::lsp::MkdlintLanguageServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

/// Helper to create a test LSP server
async fn create_test_server() -> MkdlintLanguageServer {
    let (service, _socket) = LspService::new(MkdlintLanguageServer::new);
    service.inner().clone()
}

#[tokio::test]
async fn test_initialize_and_shutdown() {
    let server = create_test_server().await;

    // Initialize with a workspace root
    let init_params = InitializeParams {
        root_uri: Some(Url::parse("file:///test/workspace").unwrap()),
        capabilities: ClientCapabilities::default(),
        ..Default::default()
    };

    let result = server.initialize(init_params).await.unwrap();

    // Verify capabilities
    assert!(result.capabilities.text_document_sync.is_some());
    assert!(result.capabilities.code_action_provider.is_some());
    assert!(result.server_info.is_some());
    assert_eq!(result.server_info.unwrap().name, "mkdlint");

    // Shutdown
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_did_open_and_close() {
    let server = create_test_server().await;

    // Initialize
    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();

    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Test\n\nTrailing spaces:   \n".to_string(),
            },
        })
        .await;

    // Wait for async processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Close document
    server
        .did_close(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        })
        .await;
}

#[tokio::test]
async fn test_did_change_with_debouncing() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Test\n".to_string(),
            },
        })
        .await;

    // Make rapid changes (should be debounced)
    for i in 2..10 {
        server
            .did_change(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: i,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: format!("# Test {}\n", i),
                }],
            })
            .await;

        // Small delay between changes
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Wait for debounce to settle
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

    // Test passed if no crashes occurred
}

#[tokio::test]
async fn test_did_save_bypasses_debounce() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "#Bad\n".to_string(),
            },
        })
        .await;

    // Save should trigger immediate lint (bypass debounce)
    server
        .did_save(DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            text: None,
        })
        .await;

    // Small delay to allow save processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test passed if no crashes occurred
}

#[tokio::test]
async fn test_code_action_returns_actions() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document with fixable issues
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "#No space\nTrailing:   \n".to_string(),
            },
        })
        .await;

    // Wait for lint to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Request code actions
    let result = server
        .code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 2,
                    character: 0,
                },
            },
            context: CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // Should have code actions (at least "Fix All")
    assert!(result.is_some());
}

#[tokio::test]
async fn test_execute_fix_all_command() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document with fixable issues
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "#Bad\n#AlsoBad\n".to_string(),
            },
        })
        .await;

    // Wait for lint
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Execute fixAll command
    let result = server
        .execute_command(ExecuteCommandParams {
            command: "mkdlint.fixAll".to_string(),
            arguments: vec![serde_json::to_value(&uri).unwrap()],
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    // Should succeed (returns None but no error)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workspace_roots_from_initialize() {
    let server = create_test_server().await;

    // Initialize with workspace folders
    let workspace_folders = vec![
        WorkspaceFolder {
            uri: Url::parse("file:///workspace1").unwrap(),
            name: "Workspace 1".to_string(),
        },
        WorkspaceFolder {
            uri: Url::parse("file:///workspace2").unwrap(),
            name: "Workspace 2".to_string(),
        },
    ];

    let init_params = InitializeParams {
        workspace_folders: Some(workspace_folders),
        capabilities: ClientCapabilities::default(),
        ..Default::default()
    };

    let result = server.initialize(init_params).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unknown_execute_command() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    // Execute unknown command
    let result = server
        .execute_command(ExecuteCommandParams {
            command: "mkdlint.unknownCommand".to_string(),
            arguments: vec![],
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    // Should succeed (returns None for unknown commands, just logs warning)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_hover_on_diagnostic() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document with a known error on line 1
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "#No space\n".to_string(), // MD018 - no space after hash
            },
        })
        .await;

    // Wait for lint to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Request hover on line 1
    let result = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    // Should have hover content
    assert!(result.is_some());
    let hover = result.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert_eq!(markup.kind, MarkupKind::Markdown);
            // Should contain rule ID and description
            assert!(markup.value.contains("MD018"));
            assert!(markup.value.contains("no-missing-space-atx"));
        }
        _ => panic!("Expected MarkupContent"),
    }
}

#[tokio::test]
async fn test_hover_on_clean_line() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document with no errors
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Good Heading\n".to_string(),
            },
        })
        .await;

    // Wait for lint
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Request hover on clean line
    let result = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    // Should return None for lines without errors
    assert!(result.is_none());
}

#[tokio::test]
async fn test_hover_shows_fix_availability() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document with fixable error
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "Trailing spaces   \n".to_string(), // MD009 - fixable
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let result = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    assert!(result.is_some());
    let hover = result.unwrap();
    match hover.contents {
        HoverContents::Markup(markup) => {
            // Should indicate auto-fixable with wrench emoji
            assert!(markup.value.contains("Auto-fixable") || markup.value.contains("🔧"));
        }
        _ => panic!("Expected MarkupContent"),
    }
}

#[tokio::test]
async fn test_capabilities_include_hover() {
    let server = create_test_server().await;

    let result = server
        .initialize(InitializeParams::default())
        .await
        .unwrap();

    // Should advertise hover capability
    assert!(result.capabilities.hover_provider.is_some());
}

#[tokio::test]
async fn test_did_change_watched_files_invalidates_cache() {
    use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType, FileEvent, Url};

    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Open document with an issue
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Test\nTrailing:   \n".to_string(),
            },
        })
        .await;

    // Wait for initial lint
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Simulate config file change
    let config_uri = Url::parse("file:///.markdownlint.json").unwrap();
    server
        .did_change_watched_files(DidChangeWatchedFilesParams {
            changes: vec![FileEvent {
                uri: config_uri,
                typ: FileChangeType::CHANGED,
            }],
        })
        .await;

    // Wait for re-lint to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Test passed if no crashes occurred
}

#[tokio::test]
async fn test_multiple_config_file_changes() {
    use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType, FileEvent, Url};

    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    // Open multiple documents
    for i in 1..=3 {
        let uri = Url::parse(&format!("file:///test{}.md", i)).unwrap();
        server
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri,
                    language_id: "markdown".to_string(),
                    version: 1,
                    text: format!("# Test {}\n", i),
                },
            })
            .await;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Simulate multiple config file changes
    let changes = vec![
        FileEvent {
            uri: Url::parse("file:///.markdownlint.json").unwrap(),
            typ: FileChangeType::CHANGED,
        },
        FileEvent {
            uri: Url::parse("file:///.markdownlint.yaml").unwrap(),
            typ: FileChangeType::CREATED,
        },
    ];

    server
        .did_change_watched_files(DidChangeWatchedFilesParams { changes })
        .await;

    // Wait for re-linting
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Test passed if all documents were re-linted without crashes
}

#[tokio::test]
async fn test_config_deletion_triggers_relint() {
    use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType, FileEvent, Url};

    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Test\n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Simulate config file deletion
    let config_uri = Url::parse("file:///.markdownlint.json").unwrap();
    server
        .did_change_watched_files(DidChangeWatchedFilesParams {
            changes: vec![FileEvent {
                uri: config_uri,
                typ: FileChangeType::DELETED,
            }],
        })
        .await;

    // Wait for re-lint
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Test passed - config deletion should trigger re-lint with default config
}

// ---------------------------------------------------------------------------
// documentSymbol tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_capabilities_include_document_symbol() {
    let server = create_test_server().await;

    let result = server
        .initialize(InitializeParams::default())
        .await
        .unwrap();

    assert!(
        result.capabilities.document_symbol_provider.is_some(),
        "Server should advertise documentSymbol capability"
    );
}

#[tokio::test]
async fn test_document_symbol_flat_headings() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Title\n\n## Section A\n\n## Section B\n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let result = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    assert!(result.is_some());
    match result.unwrap() {
        DocumentSymbolResponse::Nested(symbols) => {
            // Top level: "Title" with children "Section A" and "Section B"
            assert_eq!(symbols.len(), 1, "Should have one top-level h1");
            assert_eq!(symbols[0].name, "Title");
            let children = symbols[0].children.as_ref().unwrap();
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].name, "Section A");
            assert_eq!(children[1].name, "Section B");
        }
        _ => panic!("Expected nested document symbols"),
    }
}

#[tokio::test]
async fn test_document_symbol_nested_headings() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Root\n\n## Child\n\n### Grandchild\n\n## Child 2\n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let result = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    match result.unwrap() {
        DocumentSymbolResponse::Nested(symbols) => {
            assert_eq!(symbols.len(), 1);
            assert_eq!(symbols[0].name, "Root");
            let children = symbols[0].children.as_ref().unwrap();
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].name, "Child");
            // Grandchild should be nested under Child
            let grandchildren = children[0].children.as_ref().unwrap();
            assert_eq!(grandchildren.len(), 1);
            assert_eq!(grandchildren[0].name, "Grandchild");
            assert_eq!(children[1].name, "Child 2");
        }
        _ => panic!("Expected nested document symbols"),
    }
}

#[tokio::test]
async fn test_document_symbol_empty_document() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "Just some text, no headings.\n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let result = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    match result.unwrap() {
        DocumentSymbolResponse::Nested(symbols) => {
            assert!(symbols.is_empty(), "No headings = no symbols");
        }
        _ => panic!("Expected nested document symbols"),
    }
}

#[tokio::test]
async fn test_document_symbol_skips_code_blocks() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Real Heading\n\n```\n# Not a heading\n```\n\n## Another\n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let result = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    match result.unwrap() {
        DocumentSymbolResponse::Nested(symbols) => {
            assert_eq!(symbols.len(), 1, "Should have one top-level heading");
            assert_eq!(symbols[0].name, "Real Heading");
            let children = symbols[0].children.as_ref().unwrap();
            assert_eq!(children.len(), 1);
            assert_eq!(children[0].name, "Another");
        }
        _ => panic!("Expected nested document symbols"),
    }
}

#[tokio::test]
async fn test_document_symbol_detail_shows_level() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "## Level 2\n\n### Level 3\n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let result = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    match result.unwrap() {
        DocumentSymbolResponse::Nested(symbols) => {
            assert_eq!(symbols[0].detail, Some("h2".to_string()));
            let children = symbols[0].children.as_ref().unwrap();
            assert_eq!(children[0].detail, Some("h3".to_string()));
        }
        _ => panic!("Expected nested document symbols"),
    }
}

// ---------------------------------------------------------------------------
// Additional hover/diagnostic edge case tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_hover_multiple_errors_same_line() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    // Line with multiple errors: no space after hash AND trailing whitespace
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "#No space and trailing   \n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let result = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    assert!(result.is_some());
    match result.unwrap().contents {
        HoverContents::Markup(markup) => {
            // Should contain multiple rule sections separated by ---
            assert!(markup.value.contains("MD018"), "Should contain MD018");
            assert!(markup.value.contains("MD009"), "Should contain MD009");
        }
        _ => panic!("Expected MarkupContent"),
    }
}

#[tokio::test]
async fn test_hover_on_unknown_document() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    // Hover on a document we never opened
    let uri = Url::parse("file:///nonexistent.md").unwrap();
    let result = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    assert!(result.is_none(), "Hover on unknown doc should return None");
}

#[tokio::test]
async fn test_code_action_on_clean_document() {
    let server = create_test_server().await;

    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    server.initialized(InitializedParams {}).await;

    let uri = Url::parse("file:///test.md").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: "# Clean Document\n\nNo issues here.\n".to_string(),
            },
        })
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let result = server
        .code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri },
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 2,
                    character: 0,
                },
            },
            context: CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // Clean document should have no code actions
    assert!(
        result.is_none(),
        "Clean document should not produce code actions"
    );
}

// ── Completion tests ──────────────────────────────────────────────────────────

async fn open_doc(server: &MkdlintLanguageServer, uri: &Url, content: &str) {
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".to_string(),
                version: 1,
                text: content.to_string(),
            },
        })
        .await;
}

#[tokio::test]
async fn test_completion_capability_declared() {
    let server = create_test_server().await;
    let result = server
        .initialize(InitializeParams::default())
        .await
        .unwrap();
    assert!(
        result.capabilities.completion_provider.is_some(),
        "Server should declare completion_provider capability"
    );
}

#[tokio::test]
async fn test_completion_inside_ial_returns_items() {
    let server = create_test_server().await;
    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();

    let uri = Url::parse("file:///test/doc.md").unwrap();
    // Cursor is at end of "{: " on line 2 (0-based line 2)
    let content = "# Heading\n\n{: \n";
    open_doc(&server, &uri, content).await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 3,
                }, // after "{: "
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    let items = match result {
        Some(CompletionResponse::Array(items)) => items,
        _ => panic!("Expected CompletionResponse::Array"),
    };

    assert!(
        !items.is_empty(),
        "Should return completion items inside IAL"
    );

    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"#"), "Should include # (ID selector)");
    assert!(labels.contains(&"."), "Should include . (class selector)");
    assert!(labels.contains(&"id"), "Should include id attribute");
    assert!(labels.contains(&"class"), "Should include class attribute");
}

#[tokio::test]
async fn test_completion_outside_ial_returns_none() {
    let server = create_test_server().await;
    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();

    let uri = Url::parse("file:///test/doc2.md").unwrap();
    let content = "# Heading\n\nSome paragraph text here.\n";
    open_doc(&server, &uri, content).await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 10,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(
        result.is_none(),
        "Should not return completions outside an IAL"
    );
}

#[tokio::test]
async fn test_completion_after_closed_ial_returns_none() {
    let server = create_test_server().await;
    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();

    let uri = Url::parse("file:///test/doc3.md").unwrap();
    // Cursor is after the closing `}` — no completions expected
    let content = "# Heading\n\n{: #my-id}\n";
    open_doc(&server, &uri, content).await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 11,
                }, // after "}"
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(
        result.is_none(),
        "Should not return completions after a closed IAL"
    );
}

#[tokio::test]
async fn test_completion_filters_by_typed_prefix() {
    let server = create_test_server().await;
    server
        .initialize(InitializeParams::default())
        .await
        .unwrap();

    let uri = Url::parse("file:///test/doc4.md").unwrap();
    // User has typed "ar" after "{: " — should filter to aria-* attributes
    let content = "# Heading\n\n{: ar\n";
    open_doc(&server, &uri, content).await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 5,
                }, // after "{: ar"
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    let items = match result {
        Some(CompletionResponse::Array(items)) => items,
        _ => panic!("Expected CompletionResponse::Array"),
    };

    // All returned items should start with "ar"
    for item in &items {
        assert!(
            item.label.starts_with("ar"),
            "Filtered item '{}' should start with 'ar'",
            item.label
        );
    }
    assert!(!items.is_empty(), "Should have at least aria-* matches");
}
