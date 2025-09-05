
#[cfg(test)]
mod server_tests {
    use crate::{run_server, ServerConfig};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_server_startup() {
        let config = ServerConfig::default();
        assert!(run_server(config).await.is_ok());
    }

    #[tokio::test]
    async fn test_connection_handling() {
        let config = ServerConfig::default();
        let server = run_server(config);
        tokio::spawn(server);

        // 接続テスト
        let stream = TcpStream::connect("127.0.0.1:25565").await;
        assert!(stream.is_ok());
    }
}
#[cfg(test)]
mod tests {
    use crate::net::protocol::codec::PacketCodec;
    use crate::net::protocol::handshake::HandshakePacket;
    use crate::net::protocol::PacketState;
    use crate::utils::config::VarIntConfig;
    use crate::varint::{OptimizedVarInt, VarIntEncoder, VarIntError};
    use bytes::BytesMut;

    #[test]
    fn test_error_handling() {
        let config = VarIntConfig::default();
        let varint = OptimizedVarInt::new(config);
        let mut buf = BytesMut::new();

        // エラーケースのテスト
        let result = varint.write_string(&mut buf, "a".repeat(i32::MAX as usize + 1).as_str());
        assert!(matches!(result, Err(VarIntError::ValueTooLarge)));
    }
    #[test]
    fn test_handshake_packet() {
        let original_packet = HandshakePacket {
            protocol_version: 754,
            server_address: "localhost".to_string(),
            server_port: 25565,
            next_state: PacketState::Login,
        };

        let mut buf = BytesMut::new();
        let codec = PacketCodec::new(1024);

        // パケットのエンコード
        codec.encode_packet(&original_packet, &mut buf).unwrap();

        // パケットのデコード
        let decoded_packet = codec.decode_packet::<HandshakePacket>(&mut buf).unwrap().unwrap();

        // 元のパケットと一致することを確認
        assert_eq!(decoded_packet.protocol_version, original_packet.protocol_version);
        assert_eq!(decoded_packet.server_address, original_packet.server_address);
        assert_eq!(decoded_packet.server_port, original_packet.server_port);
        assert_eq!(decoded_packet.next_state, original_packet.next_state);
    }

}
