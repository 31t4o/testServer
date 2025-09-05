use super::status::handle_status;
use crate::net::error::ServerError;
use crate::varint::utils::{read_i64, read_varint, write_i64, write_string, write_varint};
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;
use crate::net::error::Result;

// # Minecraft Server Implementation
//
// このクレートはMinecraftサーバーの実装を提供します。
//
// ## 主な機能
//
// - プロトコル実装
// - VarInt エンコーディング
// - メトリクス収集
//
// ## 使用例
//
// ```rust
// use testServer::start_server;
//
// #[tokio::main]
// async fn main() -> Result<()> {
//     let utils = ServerConfig::default();
//     start_server(utils).await?;
//     Ok(())
// }
// ```

/// コネクションハンドラー
/// 新しい接続を処理し、適切なプロトコル処理を行います。

pub async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf = BytesMut::new();
    let size = stream.read(&mut buf).await.map_err(ServerError::Io)?;

    if size == 0 {
        return Ok(());  // 接続が閉じられた
    }

    let mut data = BytesMut::from(&buf[..size]);

    let _len = read_varint(&mut data)
        .ok_or_else(|| ServerError::Protocol("Invalid packet length".to_string()))?;
    let packet_id = read_varint(&mut data)
        .ok_or_else(|| ServerError::Protocol("Invalid packet ID".to_string()))?;
    match packet_id {
        0x00 => {
            let _protocol_version = read_varint(&mut data)
                .ok_or_else(|| ServerError::Protocol("Invalid protocol version".to_string()))?;
            let host_len = read_varint(&mut data).ok_or("Invalid host length")? as usize;

            if data.remaining() < host_len {
                return Err(ServerError::Protocol("Insufficient data for host".to_string()));
            }

            let _host = String::from_utf8(data.split_to(host_len).to_vec())
                .map_err(|_| ServerError::Protocol("Invalid UTF-8 in host".to_string()))?;

            if data.remaining() < 2 {
                return Err(ServerError::Protocol("Insufficient data for port".to_string()));
            }

            let _port = ((data.get_u8() as u16) << 8) | data.get_u8() as u16;
            let next_state = read_varint(&mut data).ok_or("Invalid next state")?;

            match next_state {
                1 => {
                    handle_status(&mut stream).await?;
                    handle_connection_ping(stream).await?;
                }
                2 => {
                    handle_connection_login(stream).await?;
                }
                _ => return Err("Invalid next state".into()),
            }
        }
        _ => return Err("Invalid initial packet ID".into()),
    }
    
    Ok(())
}

pub async fn handle_connection_ping(mut stream: TcpStream) -> Result<()> {
    let mut buf = BytesMut::new();
    let size = stream.read(&mut buf).await.map_err(ServerError::Io)?;
    let mut data = BytesMut::from(&buf[..size]);

    let _len = read_varint(&mut data)
        .ok_or_else(|| ServerError::Protocol("Invalid packet length".to_string()))?;
    let packet_id = read_varint(&mut data)
        .ok_or_else(|| ServerError::Protocol("Invalid packet ID".to_string()))?;

    if packet_id == 0x01 {
        if let Some(ping_payload) = read_i64(&mut data) {
            let mut pong = BytesMut::new();
            write_varint(&mut pong, 0x01)?;
            write_i64(&mut pong, ping_payload)?;

            let mut packet = BytesMut::new();
            write_varint(&mut packet, pong.len() as i32)?;
            packet.extend_from_slice(&pong);

            stream.write_all(&packet).await?;
        }
    }
    Ok(())
}

pub async fn handle_connection_login(mut stream: TcpStream) -> Result<()> {
    let mut buf = BytesMut::new();
    let size = stream.read(&mut buf).await.map_err(ServerError::Io)?;
    let mut data = BytesMut::from(&buf[..size]);

    let _len = read_varint(&mut data)
        .ok_or_else(|| ServerError::Protocol("Invalid packet length".to_string()))?;
    let packet_id = read_varint(&mut data)
        .ok_or_else(|| ServerError::Protocol("Invalid packet ID".to_string()))?;

    if packet_id == 0x00 {
        let name_len = read_varint(&mut data).ok_or_else(|| ServerError::Protocol("Invalid name length".to_string()))?;
        let name = String::from_utf8(data.split_to(name_len as usize).to_vec())
            .map_err(|_| ServerError::Protocol("Invalid UTF-8 in name".to_string()))?;

        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        let mut response = BytesMut::new();
        write_varint(&mut response, 0x02)?;
        write_string(&mut response, &uuid_str)?;
        write_string(&mut response, &name)?;

        let mut packet = BytesMut::new();
        write_varint(&mut packet, response.len() as i32)?;
        packet.extend_from_slice(&response);

        stream.write_all(&packet).await?;
    }
    Ok(())
}
