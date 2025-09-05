use super::{Packet, PacketError, PacketState};
use bytes::{Buf, BytesMut};
use crate::varint::utils::{read_varint, write_varint, write_string};
use crate::net::error::Result;
use crate::net::protocol::PacketError::IncompletePacket;
use crate::ServerError;

#[derive(Debug)]
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: PacketState,
}

impl Packet for HandshakePacket {
    fn packet_id(&self) -> i32 {
        0x00 // ハンドシェイクパケットのID
    }

    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        write_varint(buf, self.protocol_version)?;
        write_string(buf, &self.server_address)?;
        buf.extend_from_slice(&self.server_port.to_be_bytes());
        write_varint(buf, self.next_state as i32)?;
        Ok(())
    }

    fn decode(buf: &mut BytesMut) -> Result<Self> {
        let protocol_version = read_varint(buf)
            .ok_or_else(|| ServerError::Protocol("プロトコルバージョンの読み取りに失敗".to_string()))?;

        let server_address = {
            let len = read_varint(buf)
                .ok_or_else(|| ServerError::Protocol("アドレス長の読み取りに失敗".to_string()))? as usize;
            if buf.len() < len {
                return Err(ServerError::Protocol(IncompletePacket.to_string()));
            }
            String::from_utf8(buf.split_to(len).to_vec())
                .map_err(|_| ServerError::Protocol("不正なUTF-8シーケンス".to_string()))?
        };

        if buf.len() < 2 {
            return Err(ServerError::Protocol(IncompletePacket.to_string()));
        }
        let server_port = u16::from_be_bytes([buf[0], buf[1]]);
        buf.advance(2);

        let next_state = PacketState::from_i32(read_varint(buf)
            .ok_or_else(|| ServerError::Protocol("次の状態の読み取りに失敗".to_string()))?)
            .ok_or_else(|| ServerError::Protocol("不正な次の状態".to_string()))?;

        Ok(HandshakePacket {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }
}