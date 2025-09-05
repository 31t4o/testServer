pub mod net;
pub mod varint;
pub mod utils;

mod logging;
mod test;
mod game;

use tokio::io;
pub use utils::config::ServerConfig;
pub use net::error::{Result, ServerError};
pub use logging::setup_logging;
pub use net::start_server;

/// サーバーのメインエントリーポイント
pub async fn run_server(config: ServerConfig) -> io::Result<()> {
    if let Err(e) = setup_logging() {
        eprintln!("Warning: Failed to setup logging: {}", e);
    }
    start_server(config).await
}
