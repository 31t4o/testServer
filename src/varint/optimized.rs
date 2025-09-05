use crate::utils::config::VarIntConfig;
use crate::varint::Result;
use bytes::BytesMut;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::varint::{VarIntEncoder, VarIntError};

pub struct OptimizedVarInt {
    config: Arc<VarIntConfig>,
    encode_cache: Arc<RwLock<Vec<BytesMut>>>,
    quick_lookup: Arc<Vec<BytesMut>>,
}

impl OptimizedVarInt {
    fn init_varint(buf: &mut BytesMut, mut value: i32) -> Result<()> {
        loop {
            let mut temp: u8 = (value & 0b0111_1111) as u8;
            value >>= 7;

            if value != 0 {
                temp |= 0b1000_0000;
            }

            buf.extend_from_slice(&[temp]);

            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    pub fn new(config: VarIntConfig) -> Self {
        let config = Arc::new(config);

        // クイックルックアップテーブルの初期化
        let mut quick_lookup = Vec::with_capacity(256); // デフォルトサイズを使用
        for i in 0..256 {
            let mut buf = BytesMut::new();
            Self::init_varint(&mut buf, i as i32);
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
}

impl VarIntEncoder for OptimizedVarInt {
    fn write_varint(&self, buf: &mut BytesMut, value: i32) -> Result<()> {
        if value >= 0 && value < 256 {
            buf.extend_from_slice(&self.quick_lookup[value as usize]);
            Ok(())
        }else {Self::init_varint(buf, value);
            Ok(())
        }
    }

    fn read_varint(&self, buf: &mut BytesMut) -> Option<i32> {
        // 既存のコードをそのまま使用
        let mut result = 0;
        let mut shift = 0;

        loop {
            if buf.is_empty() {
                return None;
            }
            let byte = buf.split_to(1)[0];
            result |= ((byte & 0b0111_1111) as i32) << shift;
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                break;
            }
            if shift >= 32 {
                return None;
            }
        }
        Some(result)
    }

    fn write_varint_batch(&self, values: &[i32], buf: &mut BytesMut) -> Result<()> {
        for &value in values {
            self.write_varint(buf, value)?;
        }
        Ok(())
    }

    fn write_i64(&self, buf: &mut BytesMut, mut value: i64) -> Result<()> {
        loop {
            let mut temp: u8 = (value & 0b0111_1111) as u8;
            value >>= 7;
            if value != 0 {
                temp |= 0b1000_0000;
            }
            buf.extend_from_slice(&[temp]);
            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    fn read_i64(&self, buf: &mut BytesMut) -> Option<i64> {
        // 既存のコードをそのまま使用
        let mut result: i64 = 0;
        let mut shift = 0;

        loop {
            if buf.is_empty() {
                return None;
            }
            let byte = buf.split_to(1)[0];
            result |= ((byte & 0b0111_1111) as i64) << shift;
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                break;
            }
            if shift >= 64 {
                return None;
            }
        }
        Some(result)
    }

    fn write_string(&self, buf: &mut BytesMut, value: &str) -> Result<()> {
        if value.len() > i32::MAX as usize {
            return Err(VarIntError::ValueTooLarge);
        }
        self.write_varint(buf, value.len() as i32)?;
        buf.extend_from_slice(value.as_bytes());
        Ok(())
    }
    fn read_string(&self, buf: &mut BytesMut) -> Option<String> {
        let len = self.read_varint(buf)? as usize;
        if buf.len() < len {
            return None;
        }
        let bytes = buf.split_to(len);
        String::from_utf8(bytes.to_vec()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_varint() {
        let config = VarIntConfig {
            cache_size: 1024,
            batch_size: 100,
        };
        let varint = OptimizedVarInt::new(config);

        // 基本的なエンコード/デコードテスト
        let mut buf = BytesMut::new();
        varint.write_varint(&mut buf, 300);
        assert_eq!(varint.read_varint(&mut buf), Some(300));

        // クイックルックアップテーブルのテスト
        let mut buf = BytesMut::new();
        varint.write_varint(&mut buf, 255);
        assert_eq!(varint.read_varint(&mut buf), Some(255));

        // 大きな値のテスト
        let mut buf = BytesMut::new();
        varint.write_varint(&mut buf, i32::MAX);
        assert_eq!(varint.read_varint(&mut buf), Some(i32::MAX));
    }
}