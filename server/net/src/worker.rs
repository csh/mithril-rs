use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use mithril_core::net::{
    cast_packet,
    packets::{
        HandshakeAttemptConnect, HandshakeConnectResponse, HandshakeExchangeKey, HandshakeHello,
    },
    Packet, PacketType, RunescapeCodec,
};

enum Stage {
    Login,
    Authenticated,
}

struct Worker {
    framed: Framed<TcpStream, RunescapeCodec>,
    ip: SocketAddr,
    stage: Stage,
    listener_tx: crossbeam::Sender<()>,
}

pub async fn run_worker(
    stream: TcpStream,
    ip: SocketAddr,
    listener_tx: crossbeam::Sender<()>,
    listener_rx: crossbeam::Receiver<()>,
) {
    let framed = Framed::new(stream, RunescapeCodec::new());

    let mut worker = Worker {
        framed,
        ip,
        stage: Stage::Login,
        listener_tx,
    };

    match run_worker_impl(&mut worker).await {
        Ok(()) => String::from("disconnected"),
        Err(e) => format!("{}", e),
    };
}

async fn run_worker_impl(worker: &mut Worker) -> anyhow::Result<()> {
    loop {
        let received_message = worker.framed.next().await;
        let read_result = received_message.ok_or_else(|| anyhow::anyhow!("disconnected"))?;
        let packet = read_result?;
        handle_packet(worker, packet).await?;
        tokio::task::yield_now().await;
    }
}

async fn handle_packet(worker: &mut Worker, packet: Box<dyn Packet>) -> anyhow::Result<()> {
    match worker.stage {
        Stage::Login => {
            handle_handshake_packet(worker, packet).await;
        }
        Stage::Authenticated => {
            handle_game_packet(worker, packet).await;
        }
    }
    Ok(())
}

async fn handle_game_packet(worker: &mut Worker, packet: Box<dyn Packet>) {
//    todo!("implement game packet logic");
}

async fn handle_handshake_packet(worker: &mut Worker, packet: Box<dyn Packet>) {
    match packet.get_type() {
        PacketType::HandshakeHello => {
            handle_handshake(worker, cast_packet::<HandshakeHello>(packet)).await
        }
        PacketType::HandshakeAttemptConnect => {
            handle_login_attempt(worker, cast_packet::<HandshakeAttemptConnect>(packet)).await
        }
        _ => panic!("unexpected handshake packet"),
    }
    .unwrap();
}

async fn handle_handshake(worker: &mut Worker, handshake: HandshakeHello) -> anyhow::Result<()> {
    worker
        .framed
        .send(Box::new(HandshakeExchangeKey::default()))
        .await?;
    Ok(())
}

async fn handle_login_attempt(
    worker: &mut Worker,
    connect_attempt: HandshakeAttemptConnect,
) -> anyhow::Result<()> {
    use mithril_core::net::packets::LoginResponse;

    worker.framed.codec_mut().set_isaac_keys(
        connect_attempt.server_isaac_key,
        connect_attempt.client_isaac_key,
    );
    worker
        .framed
        .send(Box::new(HandshakeConnectResponse(LoginResponse::Success)))
        .await?;
    worker.framed.codec_mut().advance_stage();
    worker.stage = Stage::Authenticated;
    Ok(())
}
