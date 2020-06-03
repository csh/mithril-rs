use std::net::SocketAddr;

use derivative::Derivative;
use mithril_core::net::Packet;
use parking_lot::Mutex;

use specs::{Component, VecStorage};

/// Marker that indicates an entity is networked, allows for sending data back to the client.
pub struct Network {
    pub rx: Mutex<flume::Receiver<WorkerToServerMessage>>,
    pub tx: flume::Sender<ServerToWorkerMessage>,
    pub ip: SocketAddr,
}

impl Network {
    pub fn send<P>(&self, packet: P)
    where
        P: 'static + Packet,
    {
        self.send_boxed(Box::new(packet));
    }

    pub fn send_boxed(&self, packet: Box<dyn Packet>) {
        let _ = self.tx.send(ServerToWorkerMessage::Dispatch { packet });
    }
}

impl Component for Network {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub enum AuthenticationResult {
    Success,
    Failure,
    Timeout,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum ServerToWorkerMessage {
    Authenticated(AuthenticationResult),
    Dispatch {
        #[derivative(Debug = "ignore")]
        packet: Box<dyn Packet>,
    },
}

#[derive(Debug)]
pub enum WorkerToServerMessage {
    Disconnect { reason: String },
}
