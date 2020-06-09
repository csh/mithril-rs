use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::prelude::*;
use parking_lot::Mutex;
use specs::Entity;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use mithril_core::net::{
    cast_packet,
    packets::{
        HandshakeAttemptConnect, HandshakeConnectResponse, HandshakeExchangeKey, HandshakeHello,
    },
    Packet, PacketStage, PacketType, RunescapeCodec,
};
use mithril_server_types::{AuthenticationResult, ServerToWorkerMessage, WorkerToServerMessage};

use crate::{ListenerToServerMessage, NewClient, ServerToListenerMessage};
use mithril_server_packets::Packets;

struct Worker {
    framed: Framed<TcpStream, RunescapeCodec>,
    ip: SocketAddr,
    stage: PacketStage,
    listener_tx: flume::Sender<ListenerToServerMessage>,
    server_tx: flume::Sender<ServerToWorkerMessage>,
    rx: flume::Receiver<ServerToWorkerMessage>,
    server_rx: Option<flume::Receiver<WorkerToServerMessage>>,
    tx: flume::Sender<WorkerToServerMessage>,
    entity: Entity,
    packets: Arc<Packets>,
}

pub async fn run_worker(
    stream: TcpStream,
    ip: SocketAddr,
    listener_tx: flume::Sender<ListenerToServerMessage>,
    listener_rx: Arc<Mutex<flume::Receiver<ServerToListenerMessage>>>,
    packets: Arc<Packets>,
) {
    let (server_tx, rx) = flume::unbounded();
    let (tx, server_rx) = flume::unbounded();

    let entity = create_player_entity(&listener_tx, &mut *listener_rx.lock());
    let framed = Framed::new(stream, RunescapeCodec::new());

    let mut worker = Worker {
        framed,
        ip,
        stage: PacketStage::Handshake,
        listener_tx,
        server_tx,
        rx,
        server_rx: Some(server_rx),
        tx,
        entity,
        packets,
    };

    let reason = match run_worker_impl(&mut worker).await {
        Ok(()) => String::from("disconnected"),
        Err(e) => format!("{}", e),
    };

    // Server is not aware of the connection, request the entity destruction via the listener channel
    if worker.server_rx.is_some() {
        let _ = worker
            .listener_tx
            .send(ListenerToServerMessage::DestroyEntity(worker.entity));
    } else {
        let _ = worker.tx.send(WorkerToServerMessage::Disconnect { reason });
    }
}

fn create_player_entity(
    tx: &flume::Sender<ListenerToServerMessage>,
    rx: &mut flume::Receiver<ServerToListenerMessage>,
) -> Entity {
    let _ = tx.send(ListenerToServerMessage::CreateEntity);
    match rx.recv().expect("disconnect") {
        ServerToListenerMessage::EntityCreated(entity) => entity,
    }
}

async fn run_worker_impl(worker: &mut Worker) -> anyhow::Result<()> {
    loop {
        let received_message = worker.rx.next();
        let received_packet = worker.framed.next();
        let select = futures::future::select(received_message, received_packet);
        match select.await {
            future::Either::Left((message_opt, _)) => {
                if let Some(message) = message_opt {
                    handle_worker_message(worker, message).await?;
                }
            }
            future::Either::Right((packet_opt, _)) => {
                if let Some(packet_res) = packet_opt {
                    handle_packet(worker, packet_res?).await?;
                } else {
                    anyhow::bail!("disconnected")
                }
            }
        }
        tokio::task::yield_now().await;
    }
}

async fn handle_worker_message(
    worker: &mut Worker,
    message: ServerToWorkerMessage,
) -> anyhow::Result<()> {
    match message {
        ServerToWorkerMessage::Authenticated(_) => panic!("should only be handled during login"),
        ServerToWorkerMessage::Dispatch { packet } => worker.framed.send(packet).await,
    }
}

async fn handle_packet(worker: &mut Worker, packet: Box<dyn Packet>) -> anyhow::Result<()> {
    match worker.stage {
        PacketStage::Handshake => handle_handshake_packet(worker, packet).await,
        PacketStage::Gameplay => handle_game_packet(worker, packet).await,
    }
}

async fn handle_game_packet(worker: &mut Worker, packet: Box<dyn Packet>) -> anyhow::Result<()> {
    worker.packets.push(worker.entity, packet);
    Ok(())
}

async fn handle_handshake_packet(
    worker: &mut Worker,
    packet: Box<dyn Packet>,
) -> anyhow::Result<()> {
    match packet.get_type() {
        PacketType::HandshakeHello => {
            handle_handshake(worker, cast_packet::<HandshakeHello>(packet)).await
        }
        PacketType::HandshakeAttemptConnect => {
            handle_login_attempt(worker, cast_packet::<HandshakeAttemptConnect>(packet)).await
        }
        _ => panic!("unexpected handshake packet"),
    }
}

async fn handle_handshake(worker: &mut Worker, _handshake: HandshakeHello) -> anyhow::Result<()> {
    worker
        .framed
        .send(Box::new(HandshakeExchangeKey::default()))
        .await
}

async fn handle_login_attempt(
    worker: &mut Worker,
    connect_attempt: HandshakeAttemptConnect,
) -> anyhow::Result<()> {
    use mithril_core::net::packets::LoginResponse;
    let new_client = NewClient {
        ip: worker.ip,
        username: connect_attempt.username,
        password: connect_attempt.password,
        entity: worker.entity,
        tx: worker.server_tx.clone(),
        rx: worker.server_rx.take().unwrap(),
    };

    let _ = worker
        .listener_tx
        .send(ListenerToServerMessage::Authenticate(new_client));

    let authenticated = worker.rx.recv_timeout(Duration::from_secs(5)).unwrap_or(
        ServerToWorkerMessage::Authenticated(AuthenticationResult::Timeout),
    );

    let result = match authenticated {
        ServerToWorkerMessage::Authenticated(result) => result,
        _ => panic!("unexpected message at this time"),
    };

    match result {
        AuthenticationResult::Failure => {
            worker
                .framed
                .send(Box::new(HandshakeConnectResponse(
                    LoginResponse::InvalidCredentials,
                )))
                .await?;
        }
        AuthenticationResult::Timeout => {
            worker
                .framed
                .send(Box::new(HandshakeConnectResponse(
                    LoginResponse::OfflineAuthServer,
                )))
                .await?;
        }
        AuthenticationResult::Success => {
            worker.framed.codec_mut().set_isaac_keys(
                connect_attempt.server_isaac_key,
                connect_attempt.client_isaac_key,
            );
            worker
                .framed
                .send(Box::new(HandshakeConnectResponse(LoginResponse::Success)))
                .await?;
            worker.framed.codec_mut().advance_stage();
            worker.stage = PacketStage::Gameplay;
        }
    }
    Ok(())
}
