use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VarIntConfig {
    pub cache_size: usize,
    pub quick_lookup_size: usize,
    pub thread_pool_size: usize,
    pub metrics_enabled: bool,
    pub metrics_interval_secs: u64,
}

impl Default for VarIntConfig {
    fn default() -> Self {
        Self {
            cache_size: 1024 * 1024,  // 1M エントリ
            quick_lookup_size: 128,    // 頻出値
            thread_pool_size: num_cpus::get(),
            metrics_enabled: true,
            metrics_interval_secs: 60,
        }
    }
}