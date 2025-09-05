use crate::net::protocol::Packet;
use crate::varint::utils::write_string;
use crate::ServerError;
use bytes::BytesMut;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LoginDisconnect {
    pub reason_json: String,
}

impl Packet for LoginDisconnect {
    fn packet_id(&self) -> i32 {
        0x00
    }

    fn encode(&self, buf: &mut BytesMut) -> Result<(), ServerError> {
        write_string(buf, &self.reason_json)?;
        Ok(())
    }

    fn decode(_buf: &mut BytesMut) -> Result<Self, ServerError> {
        Err(ServerError::Protocol("LoginDisconnect は送信用のみです".into()))
    }
}
