use crate::util::RunescapeBuf;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use super::packets::{
    Packet, cast_packet,
    handshake::{
        HandshakeHello, HandshakeAttemptConnect, HandshakeExchangeKey, HandshakeConnectResponse
    },
};

#[derive(Debug)]
enum Stage {
    AwaitingHandshake,
    AwaitingAuth,
    Finished,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum LoginResult {
    RetryCount,
    Handshake,
    Retry,
    Success,
    InvalidCredentials,
    AccountDisabled,
    AlreadyLoggedIn,
    GameUpdate,
    WorldFull,
    OfflineAuthServer,
    ThrottleAddress,
    SessionBad,
    SessionRejected,
    MembersWorld,
    LoginIncomplete,
    ServerUpdate,
    Reconnect,
    ThrottleAuth,
    MembersArea,
    InvalidAuthServer,
    ProfileTransfer,
}

impl From<LoginResult> for u8 {
    fn from(result: LoginResult) -> Self {
        match result {
            LoginResult::Handshake => 0,
            LoginResult::Retry => 1,
            LoginResult::Success => 2,
            LoginResult::InvalidCredentials => 3,
            LoginResult::AccountDisabled => 4,
            LoginResult::AlreadyLoggedIn => 5,
            LoginResult::GameUpdate => 6,
            LoginResult::WorldFull => 7,
            LoginResult::OfflineAuthServer => 8,
            LoginResult::ThrottleAddress => 9,
            LoginResult::SessionBad => 10,
            LoginResult::SessionRejected => 11,
            LoginResult::MembersWorld => 12,
            LoginResult::LoginIncomplete => 13,
            LoginResult::ServerUpdate => 14,
            LoginResult::Reconnect => 15,
            LoginResult::ThrottleAuth => 16,
            LoginResult::MembersArea => 17,
            LoginResult::InvalidAuthServer => 20,
            LoginResult::ProfileTransfer => 21,
            LoginResult::RetryCount => 255,
        }
    }
}

pub enum Action {
    SendPacket(Box<dyn Packet>),
    Disconnect(LoginResult),
    SetIsaac(u64, u64),
    Authenticate(String, String),
}

pub struct LoginHandler {
    stage: Stage,
    action_queue: Vec<Action>,
}

impl LoginHandler {
    pub fn new() -> Self {
        Self {
            stage: Stage::AwaitingHandshake,
            action_queue: Vec::new(),
        }
    }

    pub async fn handle_packet(&mut self, packet: Box<dyn Packet>) {
        match self.stage {
            Stage::AwaitingHandshake => self.handle_name_packet(cast_packet::<HandshakeHello>(packet)),
            Stage::AwaitingAuth => self.handle_connect_attempt(cast_packet::<HandshakeAttemptConnect>(packet)),
            Stage::Finished => panic!("Login decoder already finished running"),
        }
    }

    fn handle_name_packet(&mut self, packet: HandshakeHello) {
        self.action_queue.push(Action::SendPacket(Box::new(HandshakeExchangeKey::default())));
        self.stage = Stage::AwaitingAuth;
    }

    fn handle_connect_attempt(&mut self, packet: HandshakeAttemptConnect) {
        if packet.is_reconnect {
            // TODO: Implement reconnect logic
        }

        self.action_queue.push(Action::SetIsaac(packet.server_isaac_key, packet.client_isaac_key));
        self.action_queue.push(Action::Authenticate(packet.username, packet.password));
        self.stage = Stage::Finished;
    }

    pub fn actions_to_execute(&mut self) -> Vec<Action> {
        let mut new_vec = Vec::new();
        std::mem::swap(&mut new_vec, &mut self.action_queue);
        new_vec
    }
}
