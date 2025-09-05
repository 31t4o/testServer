use crate::config::VarIntConfig;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct OptimizedVarInt {
    config: Arc<VarIntConfig>,
    encode_cache: Arc<RwLock<Vec<BytesMut>>>,
    quick_lookup: Arc<Vec<BytesMut>>,
}

impl OptimizedVarInt {
    pub fn new(config: VarIntConfig) -> Self {
        let config = Arc::new(config);

        // クイックルックアップテーブルの初期化
        let mut quick_lookup = Vec::with_capacity(config.quick_lookup_size);
        for i in 0..config.quick_lookup_size {
            let mut buf = BytesMut::new();
            Self::write_varint_raw(&mut buf, i as i32);
            quick_lookup.push(buf);
        }

        // キャッシュの初期化
        let encode_cache = Arc::new(RwLock::new(
            Vec::with_capacity(config.cache_size)
        ));

        OptimizedVarInt {
            config,
            encode_cache,
            quick_lookup: Arc::new(quick_lookup),
        }
    }

    // ... 他のメソッド ...
}