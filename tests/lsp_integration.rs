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
