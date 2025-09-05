use tokio::net::TcpListener;
use super::connection::handle_connection;
use crate::utils::config::ServerConfig;

pub async fn start_server(config: ServerConfig) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(&config.listen_address).await?;
    println!("Minecraft Rust server library is listening on {}", config.listen_address);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error: {:?}", e);
            }
        });
    }
}