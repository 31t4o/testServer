use bevy::utils::thiserror;
use bytes::{Buf, BufMut, BytesMut};
use std::io;
pub use optimized::OptimizedVarInt;
use std::sync::Arc;

mod optimized;
mod metrics;

// 基本的なトレイト定義
pub trait VarIntEncoder {
    fn write_varint(&self, buf: &mut BytesMut, value: i32) -> Result<()>;
    fn read_varint(&self, buf: &mut BytesMut) -> Option<i32>;
    fn write_varint_batch(&self, values: &[i32], buf: &mut BytesMut) -> Result<()>;
    fn write_i64(&self, buf: &mut BytesMut, value: i64) -> Result<()>;
    fn read_i64(&self, buf: &mut BytesMut) -> Option<i64>;
    fn write_string(&self, buf: &mut BytesMut, value: &str) -> Result<()>;
    fn read_string(&self, buf: &mut BytesMut) -> Option<String>;
}

// グローバルインスタンス
lazy_static::lazy_static! {
    pub static ref GLOBAL_VARINT: Arc<OptimizedVarInt> = Arc::new(
        OptimizedVarInt::new(crate::utils::config::VarIntConfig::default())
    );
}

// スレッドローカルインスタンス
thread_local! {
    pub static THREAD_LOCAL_VARINT: OptimizedVarInt = 
        OptimizedVarInt::new(crate::utils::config::VarIntConfig::default());
}

// 便利な関数群
pub mod utils {
    use super::*;

    pub fn read_varint(buf: &mut BytesMut) -> Option<i32> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            if !buf.has_remaining() {
                return None;
            }

            let byte = buf.get_u8();
            result |= ((byte & 0b0111_1111) as i32) << shift;

            if (byte & 0b1000_0000) == 0 {
                break;
            }

            shift += 7;

            if shift >= 32 {
                return None;  // 整数が大きすぎる
            }
        }

        Some(result)
    }

    pub fn write_varint(buf: &mut BytesMut, mut value: i32) -> io::Result<()> {
        loop {
            let mut temp = (value & 0b0111_1111) as u8;
            value >>= 7;
            if value != 0 {
                temp |= 0b1000_0000;
            }
            buf.put_u8(temp);
            if value == 0 {
                break;
            }
        }
        Ok(())
    }


    pub fn write_varint_batch(values: &[i32], buf: &mut BytesMut) {
        GLOBAL_VARINT.write_varint_batch(values, buf).expect("test");
        // GLOBALインスタンスを使用して各値を個別に書き込む
        // for &value in values {
        //     GLOBAL_VARINT.write_varint(buf, value);
        // }
    }

    pub fn write_i64(buf: &mut BytesMut, value: i64) -> io::Result<()> {
        GLOBAL_VARINT.write_i64(buf, value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn read_i64(buf: &mut BytesMut) -> Option<i64> {
        if buf.is_empty() {
            return None;
        }
        GLOBAL_VARINT.read_i64(buf)
    }

    pub fn write_string(buf: &mut BytesMut, value: &str) -> io::Result<()> {
        if value.len() > i32::MAX as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "String too large"));
        }
        GLOBAL_VARINT.write_string(buf, value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
    pub fn read_string(buf: &mut BytesMut) -> Option<String> {
        let len = read_varint(buf)? as usize;

        if buf.len() < len {
            return None;
        }

        let bytes = buf.split_to(len);
        String::from_utf8(bytes.to_vec()).ok()
    }
}

// エラー型の定義
#[derive(Debug, thiserror::Error)]
pub enum VarIntError {
    #[error("Value too large for VarInt encoding")]
    ValueTooLarge,

    #[error("Invalid VarInt encoding")]
    InvalidEncoding,

    #[error("Buffer underflow")]
    BufferUnderflow,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// 定数定義
pub const MAX_VARINT_LENGTH: usize = 5;
pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BIT: u8 = 0x80;

// 型エイリアス
pub type Result<T> = std::result::Result<T, VarIntError>;

// プリミティブ型に対する拡張トレイト
pub trait VarIntPrimitive: Sized {
    fn to_varint(self, buf: &mut BytesMut) -> Result<()>;
    fn from_varint(buf: &mut BytesMut) -> Option<Self>;
}

impl VarIntPrimitive for i32 {
    #[inline]
    fn to_varint(self, buf: &mut BytesMut) -> Result<()> {
        GLOBAL_VARINT.write_varint(buf, self);
        Ok(())
    }

    #[inline]
    fn from_varint(buf: &mut BytesMut) -> Option<Self> {
        GLOBAL_VARINT.read_varint(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_varint_encoding() {
        let mut buf = BytesMut::new();
        GLOBAL_VARINT.write_varint(&mut buf, 300);
        assert_eq!(GLOBAL_VARINT.read_varint(&mut buf), Some(300));
    }

    #[test]
    fn test_varint_batch() {
        let values = vec![1, 2, 3, 4, 5];
        let mut output = BytesMut::new();
        utils::write_varint_batch(&values, &mut output);

        let mut decoded = Vec::new();
        while let Some(value) = GLOBAL_VARINT.read_varint(&mut output) {
            decoded.push(value);
        }
        assert_eq!(values, decoded);
    }
}