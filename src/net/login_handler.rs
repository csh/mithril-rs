use crate::util::RunescapeBuf;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    AwaitingHandshake,
    AwaitingAuth,
    Finished,
}

#[allow(dead_code)]
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

impl From<LoginResult> for Bytes {
    fn from(result: LoginResult) -> Self {
        let mut buf = BytesMut::with_capacity(1);
        let response_code = match result {
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
        };
        buf.put_u8(response_code);
        buf.freeze()
    }
}

pub enum Action {
    SendPacket(Bytes),
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

    pub async fn handle_packet(&mut self, buf: BytesMut) {
        if self.stage == Stage::Finished {
            panic!("Login decoder already finished running");
        }

        match self.stage {
            Stage::AwaitingHandshake => self.handle_name_packet(buf),
            Stage::AwaitingAuth => self.handle_auth_packet(buf),
            _ => unreachable!(),
        }
    }

    fn handle_name_packet(&mut self, mut buf: BytesMut) {
        assert_eq!(14, buf.get_u8());
        let name_hash = buf.get_u8();
        let mut response = BytesMut::with_capacity(17);
        response.put(&[0u8; 8][..]);
        response.put_u8(0);
        response.put_u64(1234);
        self.action_queue
            .push(Action::SendPacket(response.freeze()));
        self.stage = Stage::AwaitingAuth;
    }

    fn handle_auth_packet(&mut self, mut buf: BytesMut) {
        let connection_type = buf.get_u8();
        let login_length = buf.get_u8();

        if !(connection_type == 16 || connection_type == 18) {
            self.action_queue
                .push(Action::Disconnect(LoginResult::SessionRejected));
            return;
        }

        assert!(
            buf.remaining() >= login_length as usize,
            "login_length mismatch"
        );

        let is_reconnect = connection_type == 18;
        let version = 255 - buf.get_u8();
        let release = buf.get_u16();
        let memory = buf.get_u8();
        let low_mem = memory == 1;
        let crcs = (0..9).map(|_| buf.get_u32()).collect::<Vec<_>>();

        let length = buf.get_u8();
        if length != login_length - 41 {
            self.action_queue
                .push(Action::Disconnect(LoginResult::SessionRejected));
            return;
        }

        assert_eq!(10, buf.get_u8());
        let isaac_client_key = buf.get_u64();
        let isaac_server_key = buf.get_u64();
        self.action_queue
            .push(Action::SetIsaac(isaac_server_key, isaac_client_key));

        let user_id = buf.get_u32();
        let username = buf.get_rs_string();
        let password = buf.get_rs_string();
        self.action_queue
            .push(Action::Authenticate(username, password));

        // TODO: Await authentication before sending this response
        let mut response = BytesMut::new();
        response.put_u8(2); // Success, allow login
        response.put_u8(0);
        response.put_u8(0);
        self.action_queue
            .push(Action::SendPacket(response.freeze()));
        self.stage = Stage::Finished;
    }

    pub fn actions_to_execute(&mut self) -> Vec<Action> {
        let mut new_vec = Vec::new();
        std::mem::swap(&mut new_vec, &mut self.action_queue);
        new_vec
    }
}
