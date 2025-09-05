use bytes::{BytesMut, BufMut};
use std::io;
use crate::varint::utils::{read_varint, write_varint};
use crate::net::error::Result;
use crate::net::protocol::{Packet, PacketError};
use crate::ServerError;

pub struct PacketCodec {
    max_packet_size: usize,
}

impl PacketCodec {
    pub fn new(max_packet_size: usize) -> Self {
        Self { max_packet_size }
    }

    pub fn encode_packet<P: Packet>(&self, packet: &P, buf: &mut BytesMut) -> Result<()> {
        let mut packet_buf = BytesMut::new();

        // パケットIDの書き込み
        write_varint(&mut packet_buf, packet.packet_id())?;

        // パケットデータのエンコード
        packet.encode(&mut packet_buf)?;

        // パケット長の検証
        if packet_buf.len() > self.max_packet_size {
            return Err(ServerError::Protocol("パケットが大きすぎます".to_string()));
        }

        // パケット長の書き込み
        write_varint(buf, packet_buf.len() as i32)?;

        // パケットデータの書き込み
        buf.extend_from_slice(&packet_buf);

        Ok(())
    }

    pub fn decode_packet<P: Packet>(&self, buf: &mut BytesMut) -> Result<Option<P>> {
        // パケット長の読み取り
        let packet_length = match read_varint(buf) {
            Some(len) => len as usize,
            None => return Ok(None),
        };

        // パケットサイズの検証
        if packet_length > self.max_packet_size {
            return Err(ServerError::Protocol("パケットが大きすぎます".to_string()));
        }

        // 完全なパケットを受信したか確認
        if buf.len() < packet_length {
            return Ok(None);
        }

        // パケットデータの分離
        let packet_data = buf.split_to(packet_length);
        let mut packet_buf = BytesMut::from(&packet_data[..]);

        // パケットのデコード
        let packet = P::decode(&mut packet_buf)?;

        Ok(Some(packet))
    }
}