use anyhow::Context;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};

use std::net::SocketAddr;

use bytes::BytesMut;

use crate::login_handler::LoginHandler;

struct Worker {
    framed: Framed<TcpStream, BytesCodec>,
    ip: SocketAddr,
    login_handler: Option<LoginHandler>,
}

pub async fn run_worker(stream: TcpStream, ip: SocketAddr) {
    let framed = Framed::new(stream, BytesCodec::new());

    let mut worker = Worker {
        framed,
        ip,
        login_handler: Some(LoginHandler::new()),
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
        if let Some(response) = login_handler.handle_packet(packet) {
            worker
                .framed
                .send(response)
                .await
                .context("failed to send data during login sequence")?;

            if login_handler.is_finished() {
                worker.login_handler = None;
            }
        }
    } else {
        todo!("decode remaining game packets");
    }
    Ok(())
}
