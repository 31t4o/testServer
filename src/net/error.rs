use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("VarInt error: {0}")]
    VarInt(#[from] crate::varint::VarIntError),

    #[error("Configuration error: {0}")]
    Config(String),
}

// FromUtf8Error から ServerError への変換を実装
impl From<FromUtf8Error> for ServerError {
    fn from(_: FromUtf8Error) -> Self {
        ServerError::Protocol("Invalid UTF-8 encoding".to_string())
    }
}

// &str から ServerError への変換を実装
impl From<&str> for ServerError {
    fn from(error: &str) -> Self {
        ServerError::Protocol(error.to_string())
    }
}

// String から ServerError への変換を実装
impl From<String> for ServerError {
    fn from(error: String) -> Self {
        ServerError::Protocol(error)
    }
}

pub type Result<T> = std::result::Result<T, ServerError>;
