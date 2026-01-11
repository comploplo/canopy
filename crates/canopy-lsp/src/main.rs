//! Canopy LSP Server Entry Point
//!
//! Run with: `cargo run -p canopy-lsp`

use canopy_lsp::CanopyBackend;
use tower_lsp::{LspService, Server};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize logging - MUST use stderr since stdout is for LSP JSON-RPC
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("canopy_lsp=info")),
        )
        .init();

    tracing::info!("Starting Canopy LSP server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(CanopyBackend::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
