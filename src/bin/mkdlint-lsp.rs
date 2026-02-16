//! mkdlint Language Server Protocol (LSP) server
//!
//! This binary provides LSP support for mkdlint, enabling real-time
//! linting in editors like VS Code, Neovim, and others.

use mkdlint::lsp::MkdlintLanguageServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Set up logging to stderr (stdout is used for LSP communication)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    // Create stdio transport
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Create the LSP service
    let (service, socket) = LspService::new(MkdlintLanguageServer::new);

    // Run the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
