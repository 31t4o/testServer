use std::error;
use log::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // ロギングの初期化
    env_logger::init();

    // 設定の読み込み
    let config = testServer::ServerConfig::default();

    // サーバーの起動
    if let Err(e) = testServer::run_server(config).await {
        error!("サーバーの起動に失敗しました: {}", e);
        return Err(e.into());
    }

    Ok(())
}
//github commit確認用コメント
