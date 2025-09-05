use bytes::BytesMut;
use crate::net::protocol::{Packet, PacketError};
use crate::varint::utils::{read_string, write_string, write_varint};
use crate::net::login::encryption::EncryptionKeyPair;
use crate::ServerError;

#[derive(Debug, Clone)]
pub struct EncryptionRequest {
    pub server_id: String,        // 通常は空文字列
    pub public_key: Vec<u8>,      // DER形式の公開鍵
    pub verify_token: Vec<u8>,    // ランダムバイト列
}

impl EncryptionRequest {
    pub fn from_keypair(keypair: &EncryptionKeyPair) -> Self {
        EncryptionRequest {
            server_id: "".to_string(),
            public_key: keypair.public_key_der(),
            verify_token: keypair.verify_token().to_vec(),
        }
    }
}

impl Packet for EncryptionRequest {
    fn packet_id(&self) -> i32 {
        0x01
    }

    fn encode(&self, buf: &mut BytesMut) -> Result<(), ServerError> {
        write_string(buf, &self.server_id)?;
        write_varint(buf, self.public_key.len() as i32)?;
        buf.extend_from_slice(&self.public_key);

        write_varint(buf, self.verify_token.len() as i32)?;
        buf.extend_from_slice(&self.verify_token);

        Ok(())
    }

    fn decode(_buf: &mut BytesMut) -> Result<Self, ServerError> {
        Err(ServerError::Protocol("EncryptionRequest は送信用専用です".into()))
    }
}
