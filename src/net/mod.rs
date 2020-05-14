use tokio::net::TcpListener;

mod codec;
mod login_handler;
mod worker;

pub struct NetworkManager {
    pub rx: crossbeam::Receiver<()>,
    pub tx: crossbeam::Sender<()>,
}

impl NetworkManager {
    pub fn start(listener: TcpListener) -> Self {
        let (listener_tx, rx) = crossbeam::bounded(16);
        let (tx, listener_rx) = crossbeam::bounded(16);

        tokio::spawn(run_listener(listener, listener_tx, listener_rx));

        Self { rx, tx }
    }
}

async fn run_listener(
    mut listener: TcpListener,
    tx: crossbeam::Sender<()>,
    rx: crossbeam::Receiver<()>,
) {
    loop {
        let (stream, ip) = match listener.accept().await {
            Ok(res) => res,
            Err(e) => {
                log::error!("Failed to accept connection; {}", e);
                continue;
            }
        };
        log::info!("New connection from {}", ip);
        tokio::spawn(worker::run_worker(stream, ip, tx.clone(), rx.clone()));
        tokio::task::yield_now().await;
    }
}
