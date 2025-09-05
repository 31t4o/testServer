use tracing::{Level, info, warn, error};
use crate::io::Result;

pub fn setup_logging() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("ログシステムが初期化されました");
    Ok(())
}