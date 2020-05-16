use std::net::SocketAddr;

use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};

use super::codec::{RunescapeCodec, Stage};
use super::login_handler::{Action, LoginHandler};
use super::packets::Packet;
use crate::net::login_handler::LoginResult;

struct Worker {
    framed: Framed<TcpStream, RunescapeCodec>,
    ip: SocketAddr,
    login_handler: Option<LoginHandler>,
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
        login_handler: Some(LoginHandler::new()),
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
    if let Some(ref mut login_handler) = worker.login_handler {
        login_handler.handle_packet(packet).await;
        handle_login_actions(worker).await?;
    } else {
        // TODO: Push packet to server for handling
        worker.framed.send(Box::new(crate::net::packets::game::IdAssignment)).await?;
    }
    Ok(())
}

async fn handle_login_actions(worker: &mut Worker) -> anyhow::Result<()> {
    use super::packets::handshake::HandshakeConnectResponse;

    for action in worker.login_handler.as_mut().unwrap().actions_to_execute() {
        match action {
            Action::SendPacket(packet) => {
                worker.framed.send(packet).await?
            },
            Action::Disconnect(result) => {
                worker.framed.send(Box::new(HandshakeConnectResponse(result))).await?;
                worker.framed.close().await?;
                anyhow::bail!("login handler requested disconnect");
            }
            Action::SetIsaac(server_key, client_key) => {
                worker
                    .framed
                    .codec_mut()
                    .set_isaac_keys(server_key, client_key);
            }
            Action::Authenticate(_username, _password) => {
                worker.framed.send(Box::new(HandshakeConnectResponse(LoginResult::Success))).await?;
                worker.framed.codec_mut().advance_stage();
                // TODO: Communicate to server we wish to start playing
                worker.login_handler = None;
            }
        }
    }

    Ok(())
}
