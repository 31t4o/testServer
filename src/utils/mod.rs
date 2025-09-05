mod varint;

use std::fs::File;
use std::io::Read;
pub use varint::VarIntConfig;
pub mod config;

pub fn load_config() -> VarIntConfig {
    // 設定ファイルからの読み込みを試行
    if let Ok(mut file) = File::open("configs/varint.toml") {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }
    }
    // 設定ファイルがない場合はデフォルト値を使用
    VarIntConfig::default()
}