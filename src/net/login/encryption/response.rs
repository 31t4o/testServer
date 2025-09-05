use bytes::BytesMut;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use crate::net::protocol::{Packet, PacketError};
use crate::ServerError;
use crate::varint::utils::{read_varint, write_string, write_varint};
#[derive(Debug, Clone)]
pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl EncryptionResponse {
    pub fn decode_with_key(buf: &mut BytesMut, private_key: &RsaPrivateKey) -> Result<Self, PacketError> {
        let secret_len = read_varint(buf).ok_or(PacketError::DecodeError("shared_secretの長さ読み込み失敗".into()))?;
        let encrypted_secret = buf.split_to(secret_len as usize);

        let token_len = read_varint(buf).ok_or(PacketError::DecodeError("verify_tokenの長さ読み込み失敗".into()))?;
        let encrypted_token = buf.split_to(token_len as usize);

        let shared_secret = private_key.decrypt(Pkcs1v15Encrypt, &encrypted_secret)
            .map_err(|_| PacketError::DecodeError("shared_secret復号失敗".into()))?;

        let verify_token = private_key.decrypt(Pkcs1v15Encrypt, &encrypted_token)
            .map_err(|_| PacketError::DecodeError("verify_token復号失敗".into()))?;

        Ok(EncryptionResponse {
            shared_secret,
            verify_token,
        })
    }
}

impl Packet for EncryptionResponse {
    fn packet_id(&self) -> i32 {
        0x01
    }

    fn encode(&self, _buf: &mut BytesMut) -> Result<(), ServerError> {
        Err(ServerError::Protocol("EncryptionResponse は受信用専用です".into()))
    }

    fn decode(_buf: &mut BytesMut) -> Result<Self, ServerError> {
        Err(ServerError::Protocol("decode_with_key を使用してください".into()))
    }
}