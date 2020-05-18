extern crate mithril_server_net as net;
extern crate mithril_server_packets as packets;
extern crate mithril_server_types as types;

use anyhow::Context;
use legion::prelude::*;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime;
use tokio::runtime::Handle;

mod systems;

struct GameState {
    _runtime: Handle,
    shutdown_rx: crossbeam::Receiver<()>,
    world: World,
    resources: Resources,
    executor: Executor,
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

    let world = World::new();
    let mut resources = Resources::default();
    let executor = systems::build_executor();

    let packets = Arc::new(packets::Packets::default());
    let network_manager = net::NetworkManager::start(listener, Arc::clone(&packets));
    resources.insert(packets);
    resources.insert(network_manager);

    let mut state = GameState {
        _runtime: runtime,
        shutdown_rx,
        world,
        resources,
        executor,
    };

    let state = run_game_thread(state).await;

    // TODO: Shut down server gracefully

    log::info!("Mithril is shutting down");
    std::process::exit(0);
}

async fn run_game_thread(mut state: GameState) -> GameState {
    use std::panic;
    use tokio::sync::oneshot;

    let (tx, rx) = oneshot::channel();

    std::thread::Builder::new()
        .name(String::from("mithril"))
        .spawn(|| {
            let panic = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                run_loop(&mut state);
            }));

            match panic {
                Ok(_) => (),
                Err(_) => {
                    log::error!("Mithril has crashed.");
                }
            }

            tx.send(state).ok().expect("failed to exit server thread");
        })
        .expect("failed to spawn game logic thread");

    rx.await.unwrap()
}

fn run_loop(state: &mut GameState) {
    let mut loop_helper = spin_sleep::LoopHelper::builder().build_with_target_rate(20 as f64);
    loop {
        if state.shutdown_rx.try_recv().is_ok() {
            return;
        }

        loop_helper.loop_start();
        state
            .executor
            .execute(&mut state.world, &mut state.resources);
        // TODO: Implement game logic using Legion or Specs
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
