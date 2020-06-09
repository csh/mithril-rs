use std::net::SocketAddr;
use std::sync::Arc;

use futures::SinkExt;
use futures::StreamExt;
use parking_lot::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

use mithril_core::fs::CacheFileSystem;
use mithril_core::net::JaggrabCodec;

pub fn serve_jaggrab(cache: Arc<Mutex<CacheFileSystem>>) {
    tokio::spawn(async move {
        let bind_addr = "0.0.0.0:43595";
        let mut listener = match TcpListener::bind(bind_addr).await {
            Ok(listener) => listener,
            Err(cause) => {
                /*
                 * If we've made it this far, the game socket has already been bound,
                 * JAGGRAB isn't strictly required for clients to function; should
                 * process termination be made a config option for setups that are
                 * capable of functioning without JAGGRAB?
                 */
                log::error!("Failed to bind JAGGRAB listener; {}", cause);
                std::process::exit(1);
            }
        };

        log::info!("JAGGRAB is listening on {}", bind_addr);

        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(connection) => connection,
                Err(cause) => {
                    log::error!("Error accepting JAGGRAB request; {}", cause);
                    continue;
                }
            };

            let cache = Arc::clone(&cache);
            tokio::spawn(serve_request(stream, addr, cache));
        }
    });
}

async fn serve_request(stream: TcpStream, addr: SocketAddr, cache: Arc<Mutex<CacheFileSystem>>) {
    let mut framed = Framed::new(stream, JaggrabCodec);
    let file = match framed.next().await.expect("valid") {
        Ok(file) => file,
        Err(cause) => {
            log::error!("JAGGRAB request for '{}' failed; {}", addr, cause);
            return;
        }
    };

    log::debug!("{} requested {:?} using JAGGRAB", addr, file);
    let file = { cache.lock().get_file(0, file as _) };

    match file {
        Ok(data) => {
            let _ = framed.send(data).await;
        }
        Err(cause) => {
            log::error!(
                "JAGGRAB encountered an error whilst reading the cache; {}",
                cause
            );
        }
    }
}
