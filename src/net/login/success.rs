use crate::net::protocol::Packet;
use crate::varint::utils::write_string;
use crate::ServerError;
use bytes::BytesMut;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
}

impl Packet for LoginSuccess {
    fn packet_id(&self) -> i32 {
        0x02
    }

    fn encode(&self, buf: &mut BytesMut) -> Result<(), ServerError> {
        write_string(buf, &self.uuid.hyphenated().to_string())?;
        write_string(buf, &self.username)?;
        Ok(())
    }

    fn decode(_buf: &mut BytesMut) -> Result<LoginSuccess, ServerError> {
        Err(ServerError::Protocol("LoginSuccess はクライアントへの送信専用です".into()))
    }
}
