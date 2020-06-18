use mithril_core::fs::CacheFileSystem;
use mithril_jaggrab as jaggrab;
use std::sync::Arc;

fn main() {
    simple_logger::init().unwrap();

    let cache = CacheFileSystem::open("cache")
        .unwrap_or_else(|e| {
            eprintln!("Failed to open cache; {}", e);
            std::process::exit(1);
        });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    if let Err(e) = jaggrab::serve_jaggrab("0.0.0.0:43595", Arc::new(pool), cache) {
        log::error!("Failed to bootstrap JAGGRAB; {}", e);
    }
}
