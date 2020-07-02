use std::net::TcpListener;
use std::time::Duration;

use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::{
    network::simulation::tcp::TcpNetworkBundle, prelude::*, utils::application_dir, Result,
};

use mithril::{
    core::{
        fs::{defs, CacheFileSystem},
        net::packets::ObjectType,
        pos::*,
    },
    net::MithrilNetworkBundle,
    player::PlayerEntityBundle,
    types::{
        auth::{AlwaysAllowStrategy, Authenticator},
        components::{StaticObject, WorldObjectData},
        CollisionDetector,
    },
};

#[cfg(feature = "jaggrab")]
fn add_jaggrab_bundle<'a, 'b>(
    game_data: GameDataBuilder<'a, 'b>,
) -> Result<GameDataBuilder<'a, 'b>> {
    let listener = TcpListener::bind("0.0.0.0:43595")?;
    listener.set_nonblocking(true)?;
    game_data.with_bundle(mithril_jaggrab::JaggrabServerBundle::new(Some(listener)))
}

#[cfg(not(feature = "jaggrab"))]
fn add_jaggrab_bundle<'a, 'b>(
    game_data: GameDataBuilder<'a, 'b>,
) -> Result<GameDataBuilder<'a, 'b>> {
    Ok(game_data)
}

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let listener = TcpListener::bind("0.0.0.0:43594")?;
    listener.set_nonblocking(true)?;

    let assets_dir = application_dir("../cache")?;

    let mut game_data = GameDataBuilder::default()
        .with_bundle(TcpNetworkBundle::new(Some(listener), 4096))?
        .with_bundle(MithrilNetworkBundle)?
        .with_bundle(PlayerEntityBundle)?;

    game_data = add_jaggrab_bundle(game_data)?;

    let mut game = Application::build(assets_dir, LoadingState::default())?
        .with_fixed_step_length(Duration::from_millis(600))
        .with_frame_limit(FrameRateLimitStrategy::Yield, 10)
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
        #[cfg(feature = "profiler")]
        thread_profiler::profile_scope!("init");

        log::info!("Initializing Mithril..");

        let cache_path = match application_dir("../cache") {
            Ok(path) => path,
            Err(e) => panic!(e),
        };

        let cache = match CacheFileSystem::open(cache_path) {
            Ok(cache) => cache,
            Err(cause) => {
                log::error!("Failed to open cache; {}", cause);
                return;
            }
        };

        match CollisionDetector::new(&cache) {
            Ok(detector) => data.world.insert(detector),
            Err(cause) => {
                log::error!("Failed to map collisions; {}", cause);
                return;
            }
        }

        let map_indices = match defs::MapIndex::load(&cache) {
            Ok(indices) => indices,
            Err(cause) => {
                log::error!("Failed to load map indices; {}", cause);
                return;
            }
        };

        data.world.register::<WorldObjectData>();

        for idx in map_indices.values() {
            let object_defs = match defs::MapObject::load(&cache, idx) {
                Ok(defs) => defs,
                Err(cause) => {
                    log::error!("Failed to load map objects; {}", cause);
                    return;
                }
            };
            for object_def in object_defs {
                let id = object_def.get_id();
                let object_type: ObjectType = object_def.get_variant().into();
                let orientation: Direction = object_def.get_orientation().into();
                // TODO: Make Position or core/fs module more consistent with data types
                let pos = Position::new_with_height(
                    idx.get_x() as i16 + object_def.get_x(),
                    idx.get_y() as i16 + object_def.get_y(),
                    object_def.get_plane(),
                )
                .expect("all cache planes should be valid");
                data.world
                    .create_entity()
                    .with(StaticObject)
                    .with(pos)
                    .with(WorldObjectData::Object {
                        id,
                        object_type,
                        orientation,
                    })
                    .build();
            }
        }

        match defs::ObjectDefinition::load(&cache) {
            Ok(defs) => data.world.insert(defs),
            Err(cause) => {
                log::error!("Failed to load object definitions; {}", cause);
                return;    
            }    
        }

        data.world.insert(cache);
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
