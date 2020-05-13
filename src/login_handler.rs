use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    AwaitingHandshake,
    AwaitingAuth,
    Finished,
}

#[allow(dead_code)]
enum LoginResult {
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
    Unknown,
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
            LoginResult::Unknown => 254,
        }
    }
}

pub struct LoginHandler {
    stage: Stage,
}

impl LoginHandler {
    pub fn new() -> Self {
        Self {
            stage: Stage::AwaitingHandshake,
        }
    }

    pub fn handle_packet(&mut self, buf: BytesMut) -> Option<Bytes> {
        if self.stage == Stage::Finished {
            panic!("Login decoder already finished running");
        }

        match self.stage {
            Stage::AwaitingHandshake => self.handle_name_packet(buf),
            Stage::AwaitingAuth => self.handle_auth_packet(buf),
            _ => unreachable!(),
        }
    }

    fn handle_name_packet(&mut self, mut buf: BytesMut) -> Option<Bytes> {
        const RESPONSE_PADDING: [u8; 8] = [0u8; 8];

        assert_eq!(14, buf.get_u8());
        let name_hash = buf.get_u8();
        log::debug!("Name hash = {}", name_hash);
        self.stage = Stage::AwaitingAuth;

        let mut response = BytesMut::with_capacity(17);
        response.put(&RESPONSE_PADDING[..]);
        response.put_u8(LoginResult::Handshake.into());
        response.put_u64(1234);
        Some(response.freeze())
    }

    fn handle_auth_packet(&mut self, mut buf: BytesMut) -> Option<Bytes> {
        let connection_type = buf.get_u8();
        let login_length = buf.get_u8();

        if !(connection_type == 16 || connection_type == 18) {
            return Some(response_helper(LoginResult::SessionRejected));
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
        let crcs = (0..9)
            .map(|_| buf.get_u32())
            .inspect(|v| log::debug!("CRC: {}", v))
            .collect::<Vec<_>>();

        let length = buf.get_u8();
        if length != login_length - 41 {
            return Some(response_helper(LoginResult::SessionRejected));
        }
        assert_eq!(10, buf.get_u8());

        let isaac_client_key = buf.get_u64();
        let isaac_server_key = buf.get_u64();
        let user_id = buf.get_u32();
        let username = get_rs_string(&mut buf);
        let password = get_rs_string(&mut buf);

        log::debug!("isaac_client_key = {}", isaac_client_key);
        log::debug!("isaac_server_key = {}", isaac_server_key);
        log::debug!("user_id = {}", user_id);
        log::debug!("username = {}", username);

        self.stage = Stage::Finished;

        let mut response = BytesMut::new();
        response.put_u8(LoginResult::Success.into());
        response.put_u8(0);
        response.put_u8(0);
        Some(response.freeze())
    }

    pub fn is_finished(&self) -> bool {
        self.stage == Stage::Finished
    }
}

fn get_rs_string(buf: &mut BytesMut) -> String {
    let mut result = String::default();
    loop {
        match buf.get_u8() {
            10 => break,
            c => result.push(char::from(c)),
        }
    }
    result
}

fn response_helper(result: LoginResult) -> Bytes {
    let mut buf = BytesMut::with_capacity(1);
    buf.put_u8(result.into());
    buf.freeze()
}
