use anyhow::Context;

use tokio::net::TcpListener;
use tokio::runtime::Handle;

mod login_handler;
mod worker;

struct GameState {
    _runtime: Handle,
    shutdown_rx: crossbeam::Receiver<()>,
}

pub fn handle_shutdown(tx: crossbeam::Sender<()>) {
    ctrlc::set_handler(move || {
        tx.send(()).expect("Failed to send CTRL-C shutdown");
    })
    .expect("Failed to set CTRL-C handler");
}

async fn run_listener(mut listener: TcpListener) {
    loop {
        let (stream, ip) = match listener.accept().await {
            Ok(res) => res,
            Err(e) => {
                log::error!("Failed to accept connection; {}", e);
                continue;
            }
        };
        log::info!("New connection from {}", ip);
        tokio::spawn(worker::run_worker(stream, ip));
    }
}

pub async fn main(runtime: Handle) {
    let (shutdown_tx, shutdown_rx) = crossbeam::bounded(1);
    handle_shutdown(shutdown_tx);

    let state = GameState {
        _runtime: runtime,
        shutdown_rx,
    };

    let bind_addr = "0.0.0.0:43594";
    let listener = match TcpListener::bind(bind_addr)
        .await
        .context("Failed to bind listener")
    {
        Ok(listener) => listener,
        Err(e) => {
            log::error!("Server failed to start: {}", e);
            std::process::exit(1);
        }
    };

    log::info!("Listening on {}", bind_addr);
    tokio::spawn(run_listener(listener));

    loop {
        if state.shutdown_rx.try_recv().is_ok() {
            return;
        }
    }
}
