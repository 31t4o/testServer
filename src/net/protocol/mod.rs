pub(crate) mod handshake;
pub(crate) mod codec;
mod login;

use bytes::BytesMut;
use uuid::Uuid;
use crate::net::error::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketState {
    Handshake = 0,
    Status = 1,
    Login = 2,
    Play = 3,
}

impl PacketState {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(PacketState::Handshake),
            1 => Some(PacketState::Status),
            2 => Some(PacketState::Login),
            3 => Some(PacketState::Play),
            _ => None,
        }
    }
}

pub trait Packet: Send + Sized {
    /// パケットのIDを返す
    fn packet_id(&self) -> i32;

    /// パケットをバイトストリームにエンコード
    fn encode(&self, buf: &mut BytesMut) -> Result<()>;

    /// バイトストリームからパケットをデコード
    fn decode(buf: &mut BytesMut) -> Result<Self>;
}

#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("無効なパケットID: {0}")]
    InvalidPacketId(i32),

    #[error("パケットデータが不完全")]
    IncompletePacket,

    #[error("デコードエラー: {0}")]
    DecodeError(String),

    #[error("エンコードエラー: {0}")]
    EncodeError(String),
}