use crate::net::protocol::Packet;
use crate::varint::utils::{read_string, write_string};
use crate::ServerError;
use bytes::BytesMut;

#[derive(Debug, Clone)]
pub struct LoginStart {
    pub username: String,
}

impl Packet for LoginStart {
    fn packet_id(&self) -> i32 {
        0x00
    }

    fn encode(&self, buf: &mut BytesMut) -> Result<(), ServerError> {
        write_string(buf, &self.username)?;
        Ok(())
    }

    fn decode(buf: &mut BytesMut) -> Result<LoginStart, ServerError> {
        let username = read_string(buf).ok_or_else(|| ServerError::Protocol("ユーザー名の読み込みに失敗".into()))?;
        Ok(Self { username })
    }
}
