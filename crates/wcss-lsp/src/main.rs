mod completions;
mod diagnostics_provider;
mod server;

use server::WcssLanguageServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| WcssLanguageServer::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;
}
