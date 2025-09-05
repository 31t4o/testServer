use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::{BytesMut, Buf, BufMut};
use crate::varint::utils::{read_varint, write_varint};
use crate::net::error::Result;

pub async fn handle_status(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0u8; 1024];
    let size = stream.read(&mut buf).await?;
    let mut data = BytesMut::from(&buf[..size]);
    let _len = read_varint(&mut data).ok_or_else(|| crate::net::error::ServerError::Protocol("Invalid length".to_string()))?;
    let packet_id = read_varint(&mut data).ok_or_else(|| crate::net::error::ServerError::Protocol("Invalid packet ID".to_string()))?;

    if packet_id == 0x00 {
        // Status Response
        let response = serde_json::json!({
            "version": { "name": "1.20.1", "protocol": 763 },
            "players": { "max": 20, "online": 0 },
            "description": { "text": "5io Test Server" }
        });

        let response_str = response.to_string();
        let mut response_buf = BytesMut::new();
        write_varint(&mut response_buf, 0x00)?;
        write_varint(&mut response_buf, response_str.len() as i32)?;
        response_buf.put_slice(response_str.as_bytes());

        let mut full_packet = BytesMut::new();
        write_varint(&mut full_packet, response_buf.len() as i32)?;
        full_packet.put(response_buf);

        stream.write_all(&full_packet).await?;
    } else if packet_id == 0x01 {
        // Ping Response
        let payload = data.get_i64();
        let mut pong_buf = BytesMut::new();
        write_varint(&mut pong_buf, 0x01)?;
        pong_buf.put_i64(payload);

        let mut full = BytesMut::new();
        write_varint(&mut full, pong_buf.len() as i32)?;
        full.put(pong_buf);

        stream.write_all(&full).await?;
    }

    Ok(())
}

