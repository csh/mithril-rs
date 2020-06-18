use std::net::TcpListener;
use std::time::Duration;

use amethyst::{
    network::simulation::tcp::TcpNetworkBundle,
    prelude::*, utils::application_dir, Result,
};

use mithril::{
    core::fs::CacheFileSystem,
    player::PlayerEntityBundle,
    net::MithrilNetworkBundle,
    types::{
        CollisionDetector,
        auth::{AlwaysAllowStrategy, Authenticator}
    },
};

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let listener = TcpListener::bind("0.0.0.0:43594")?;
    listener.set_nonblocking(true)?;

    let assets_dir = application_dir("../cache")?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TcpNetworkBundle::new(Some(listener), 4096))?
        .with_bundle(MithrilNetworkBundle)?
        .with_bundle(PlayerEntityBundle)?;

    let mut game = Application::build(assets_dir, LoadingState::default())?
        .with_fixed_step_length(Duration::from_millis(600))
        .build(game_data)?;
    game.run();
    Ok(())
}

#[derive(Default, Debug)]
pub struct LoadingState {
    loaded: bool,
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("Initializing Mithril..");

        let cache_path = match application_dir("../cache") {
            Ok(path) => path,
            Err(e) => panic!(e),
        };

        match CacheFileSystem::open(cache_path) {
            Ok(cache) => {
                data.world.insert(cache);
            }
            Err(cause) => {
                log::error!("Failed to open cache; {}", cause);
                return;
            }
        };

        let mut cache = data.world.get_mut::<CacheFileSystem>().unwrap();
        match CollisionDetector::new(&mut cache) {
            Ok(detector) => data.world.insert(detector),
            Err(cause) => {
                log::error!("Failed to map collisions; {}", cause);
                return;
            }
        }

        data.world.insert(Authenticator::new(AlwaysAllowStrategy));
        self.loaded = true;
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.loaded {
            Trans::Switch(Box::new(GameState))
        } else {
            Trans::Quit
        }
    }
}

pub struct GameState;

impl SimpleState for GameState {
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        log::info!("Mithril is ready!");
    }
}
