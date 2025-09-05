use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub listen_address: String,
    pub max_connections: usize,
    pub metrics: MetricsConfig,
    pub varint: VarIntConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub interval: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VarIntConfig {
    pub cache_size: usize,
    pub batch_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_address: "127.0.0.1:25565".to_string(),
            max_connections: 1000,
            metrics: MetricsConfig::default(),
            varint: VarIntConfig::default(),
        }
    }
}

impl Default for VarIntConfig {
    fn default() -> Self {
        Self {
            cache_size: 1024,
            batch_size: 100,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "http://localhost:9091".to_string(),
            interval: Duration::from_secs(60),
        }
    }
}