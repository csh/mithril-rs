use anyhow::Context;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};

use std::net::SocketAddr;

use bytes::BytesMut;

use super::codec::RunescapeCodec;
use super::login_handler::{Action, LoginHandler};

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
        log::debug!("{} sent {:?}", worker.ip, packet);
        handle_packet(worker, packet).await?;
        tokio::task::yield_now().await;
    }
}

async fn handle_packet(worker: &mut Worker, packet: BytesMut) -> anyhow::Result<()> {
    if let Some(ref mut login_handler) = worker.login_handler {
        login_handler.handle_packet(packet).await;
        handle_login_actions(worker).await?;
    } else {
        todo!("decode remaining game packets");
    }
    Ok(())
}

async fn handle_login_actions(worker: &mut Worker) -> anyhow::Result<()> {
    for action in worker.login_handler.as_mut().unwrap().actions_to_execute() {
        match action {
            Action::SendPacket(packet) => worker.framed.send(packet).await?,
            Action::Disconnect(result) => {
                worker.framed.send(result.into()).await?;
                anyhow::bail!("login handler requested disconnect");
            }
            Action::SetIsaac(server_key, client_key) => {
                log::info!("ISAAC client key: {}", client_key);
                log::info!("ISAAC server key: {}", server_key);
                worker
                    .framed
                    .codec_mut()
                    .set_isaac_keys(server_key, client_key);
            }
            Action::Authenticate(_username, _password) => {
                // TODO: Actually authenticate with the server via channels
                log::info!("username: {}, password: {}", _username, _password);
                worker.login_handler = None;
            }
        }
    }

    Ok(())
}
