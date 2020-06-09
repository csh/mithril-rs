extern crate mithril_server_net as net;
extern crate mithril_server_packets as packets;
extern crate mithril_server_types as types;

use anyhow::Context;
use specs::prelude::*;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime;
use tokio::runtime::Handle;
use std::panic;
use mithril_core::fs::CacheFileSystem;
use types::CollisionDetector;

mod systems;

struct GameState<'a, 'b> {
    _runtime: Handle,
    shutdown_rx: crossbeam::Receiver<()>,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

pub fn handle_shutdown(tx: crossbeam::Sender<()>) {
    ctrlc::set_handler(move || {
        tx.send(()).expect("Failed to send CTRL-C shutdown");
    })
    .expect("Failed to set CTRL-C handler");
}

pub async fn run(runtime: Handle) {
    let (shutdown_tx, shutdown_rx) = crossbeam::bounded(1);
    handle_shutdown(shutdown_tx);

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

    let cache_dir = if cfg!(debug_assertions) {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../cache")
    } else {
        "./cache"
    };
    // TODO: Implement proper error handling in core/fs module
    let mut cache = CacheFileSystem::open(cache_dir).unwrap_or_else(|_| {
                log::error!("Unable to find cache data; please place files in {}", cache_dir);
                std::process::exit(1);
        });

    let mut world = World::new();
    let mut dispatcher = systems::build_dispatcher();

    let packets = Arc::new(packets::Packets::default());
    let network_manager = net::NetworkManager::start(listener, Arc::clone(&packets));
    let collision_detector = CollisionDetector::new(&mut cache)
        .unwrap_or_else(|why| {
            log::error!("Mithril experienced an error whilst loading map data; {}", why);
            std::process::exit(1);
        });

    world.insert(cache);
    world.insert(collision_detector);
    world.insert(packets);
    world.insert(network_manager);

    dispatcher.setup(&mut world);

    let mut state = GameState {
        _runtime: runtime,
        shutdown_rx,
        world,
        dispatcher,
    };

    let panic = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        run_loop(&mut state)
    }));

    if let Err(_) = panic {
        log::error!("Mithril has crashed!");
    }

    // TODO: Shut down server gracefully

    log::info!("Mithril is shutting down");
    std::process::exit(0);
}

fn run_loop(state: &mut GameState) {
    let mut loop_helper = spin_sleep::LoopHelper::builder().build_with_target_rate(6 as f64);
    loop {
        if state.shutdown_rx.try_recv().is_ok() {
            return;
        }

        loop_helper.loop_start();
        state.dispatcher.dispatch(&mut state.world);
        state.world.maintain();
        loop_helper.loop_sleep();
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Debug).expect("Failed to init logging");

    let mut runtime = runtime::Builder::new()
        .threaded_scheduler()
        .thread_name("mithril-worker-pool")
        .enable_all()
        .build()
        .expect("failed to start tokio runtime");

    let handle = runtime.handle().clone();
    runtime.block_on(async move {
        run(handle).await;
    });
}
