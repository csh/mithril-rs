use std::sync::Arc;

use mithril_server_packets::Packets;
use mithril_server_types::{ServerToWorkerMessage, WorkerToServerMessage};

use derivative::Derivative;
use specs::Entity;
use parking_lot::Mutex;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod jaggrab_worker;
mod worker;

pub use jaggrab_worker::serve_jaggrab;

#[derive(Debug)]
pub enum ListenerToServerMessage {
    CreateEntity,
    DestroyEntity(Entity),
    Authenticate(NewClient),
}

#[derive(Debug)]
pub enum ServerToListenerMessage {
    EntityCreated(Entity),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NewClient {
    pub ip: SocketAddr,
    pub username: String,
    pub password: String,
    pub entity: Entity,

    #[derivative(Debug = "ignore")]
    pub tx: flume::Sender<ServerToWorkerMessage>,
    #[derivative(Debug = "ignore")]
    pub rx: flume::Receiver<WorkerToServerMessage>,
}

pub struct NetworkManager {
    pub rx: Mutex<flume::Receiver<ListenerToServerMessage>>,
    pub tx: flume::Sender<ServerToListenerMessage>,
}

impl NetworkManager {
    pub fn start(listener: TcpListener, packets: Arc<Packets>) -> Self {
        let (listener_tx, rx) = flume::bounded(16);
        let (tx, listener_rx) = flume::bounded(16);

        tokio::spawn(run_listener(listener, listener_tx, listener_rx, packets));

        Self {
            rx: Mutex::new(rx),
            tx,
        }
    }
}

async fn run_listener(
    mut listener: TcpListener,
    listener_tx: flume::Sender<ListenerToServerMessage>,
    listener_rx: flume::Receiver<ServerToListenerMessage>,
    packets: Arc<Packets>,
) {
    let rx = Arc::new(Mutex::new(listener_rx));
    loop {
        let (stream, ip) = match listener.accept().await {
            Ok(res) => res,
            Err(e) => {
                log::error!("Failed to accept connection; {}", e);
                continue;
            }
        };
        log::info!("New connection from {}", ip);
        tokio::spawn(worker::run_worker(
            stream,
            ip,
            listener_tx.clone(),
            Arc::clone(&rx),
            Arc::clone(&packets),
        ));
        tokio::task::yield_now().await;
    }
}
