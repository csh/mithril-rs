use std::io;
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use std::sync::Arc;

use mithril_core::fs::CacheFileSystem;
use rayon::ThreadPool;

pub fn serve_jaggrab<A: ToSocketAddrs>(
    bind_addr: A,
    thread_pool: Arc<ThreadPool>,
    cache: CacheFileSystem,
) -> io::Result<()> {
    let mut bind_addrs = bind_addr.to_socket_addrs()?;
    while let Some(bind_addr) = bind_addrs.next() {
        log::debug!("Binding JAGGRAB listener to {}", bind_addr);
        bind_listener(bind_addr, Arc::clone(&thread_pool), &cache)?;
    }
    Ok(())
}

fn bind_listener(
    bind_addr: SocketAddr,
    thread_pool: Arc<ThreadPool>,
    cache: &CacheFileSystem,
) -> io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    let worker_pool = Arc::clone(&thread_pool);
    thread_pool.install(move || loop {
        let (stream, _) = match listener.accept() {
            Ok(accept) => accept,
            Err(cause) => {
                log::error!("Failed to accept connection; {}", cause);
                continue;
            }
        };

        worker_pool.install(move || {
            if let Err(cause) = crate::serve_request(stream, cache) {
                log::error!("Failed to fulfil JAGGRAB request; {}", cause);
            }
        });
    });
    Ok(())
}
