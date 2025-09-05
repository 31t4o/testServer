use crate::net::error::Result;
use crate::net::protocol::{Packet, PacketError};
use crate::varint::utils::{read_string, write_string};
use bytes::BytesMut;
use crate::ServerError;

pub struct LoginStart {
    pub username: String,
}

impl Packet for LoginStart {
    fn packet_id(&self) -> i32 {
        0x00
    }

    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        write_string(buf, &self.username).expect("encode error");
        Ok(())
    }

    fn decode(buf: &mut BytesMut) -> Result<Self> {
        let name = read_string(buf)
            .ok_or_else(|| ServerError::Protocol("ユーザー名読み込み失敗".into()))?;
        Ok(Self { username: name })
    }
}
