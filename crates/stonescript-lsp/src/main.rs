use tower_lsp::{LspService, Server};
use stonescript_lsp::server;

mod data;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting StoneScript Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| server::Backend::new(client));
    
    Server::new(stdin, stdout, socket).serve(service).await;
}
