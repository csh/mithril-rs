use super::prelude::*;
use crate::net::login_handler::LoginResult;
use crate::util::RunescapeBuf;

#[derive(Default, Debug)]
pub struct HandshakeHello {
    pub name_hash: u8
}

impl Packet for HandshakeHello {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        assert_eq!(14, src.get_u8(), "invalid packet");
        self.name_hash = src.get_u8();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::HandshakeHello
    }
}

#[derive(Default, Debug)]
pub struct HandshakeAttemptConnect {
    pub is_reconnect: bool,
    pub version: u8,
    pub release: u16,
    pub low_memory: bool,
    pub crc: Vec<u32>,
    pub client_isaac_key: u64,
    pub server_isaac_key: u64,
    pub user_id: u32,
    pub username: String,
    pub password: String,
}

impl Packet for HandshakeAttemptConnect {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        let connection_type = src.get_u8();
        let login_length = src.get_u8();
        assert!(src.remaining() >= login_length as usize);
        self.is_reconnect = connection_type == 18;
        self.version = 255 - src.get_u8();
        self.release = src.get_u16();
        self.low_memory = src.get_u8() == 1;
        self.crc = (0..9).map(|_| src.get_u32()).collect::<Vec<u32>>();
        let remaining = src.get_u8();
        assert_eq!(remaining, login_length - 41, "malformed login packet");
        assert_eq!(10, src.get_u8());
        self.client_isaac_key = src.get_u64();
        self.server_isaac_key = src.get_u64();
        self.user_id = src.get_u32();
        self.username = src.get_rs_string();
        self.password = src.get_rs_string();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::HandshakeAttemptConnect
    }
}

#[derive(Debug)]
pub struct HandshakeExchangeKey {
    session_key: u64,
    response_code: LoginResult,
}

impl Default for HandshakeExchangeKey {
    fn default() -> Self {
        HandshakeExchangeKey {
            session_key: rand::random::<u64>(),
            response_code: LoginResult::Handshake
        }
    }
}

impl Packet for HandshakeExchangeKey {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        src.put_slice(&[0; 8]);
        src.put_u8(self.response_code.into());
        src.put_u64(self.session_key);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::HandshakeExchangeKey
    }
}

#[derive(Debug)]
pub struct HandshakeConnectResponse(pub LoginResult);

impl Packet for HandshakeConnectResponse {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        src.put_u8(self.0.into());
        src.put_u8(0);
        src.put_u8(0);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::HandshakeConnectResponse
    }
}