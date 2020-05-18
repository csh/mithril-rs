use derivative::Derivative;
use mithril_core::net::Packet;
use parking_lot::Mutex;

/// Marker that indicates an entity is networked, allows for sending data back to the client.
pub struct Networked {
    pub rx: Mutex<flume::Receiver<WorkerToServerMessage>>,
    pub tx: flume::Sender<ServerToWorkerMessage>,
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
    Disconnect,
}
