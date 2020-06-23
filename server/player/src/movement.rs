#![allow(clippy::type_complexity)]
use amethyst::{
    core::{Named, SystemDesc},
    ecs::{prelude::*, world::Index, RunningTime},
    shrev::EventChannel,
};

use ahash::AHashMap;
use mithril_core::{
    net::packets::{
        AddPlayer, Appearance, AppearanceType, EntityMovement, Equipment, Item, NpcSynchronization,
        PlayerSynchronization, PlayerUpdate, RegionChange, SyncBlocks,
    },
    pos::{Direction, Position},
};

use mithril_server_net::{
    EntityPacketEvent, GameplayEvent, MithrilTransportResource, PacketEvent, PacketEventChannel,
};
use mithril_server_types::{CollisionDetector, Pathfinder, PreviousPosition, VisiblePlayers};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Default)]
pub struct PlayerSyncSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, PlayerSyncSystem> for PlayerSyncSystemDesc {
    fn build(self, world: &mut World) -> PlayerSyncSystem {
        <PlayerSyncSystem as System<'_>>::SystemData::setup(world);
        PlayerSyncSystem
    }
}

#[derive(SystemData)]
pub struct PlayerSyncStorage<'a> {
    names: ReadStorage<'a, Named>,
    positions: ReadStorage<'a, Position>,
    previous_positions: ReadStorage<'a, PreviousPosition>,
}

pub struct PlayerSyncSystem;

impl<'a> System<'a> for PlayerSyncSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, MithrilTransportResource>,
        PlayerSyncStorage<'a>,
        WriteStorage<'a, VisiblePlayers>,
    );

    #[allow(clippy::type_complexity)]
    fn run(&mut self, (entities, mut net, sync, mut visible_players): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("player sync");

        for (entity, named, current_pos, previous, visible) in (
            &entities,
            &sync.names,
            &sync.positions,
            sync.previous_positions.maybe(),
            &mut visible_players,
        )
            .join()
        {
            type RemotePlayer<'a> = (
                Entity,
                &'a Position,
                Option<&'a PreviousPosition>,
                &'a Named,
            );

            let local: Vec<RemotePlayer<'_>> = (
                &entities,
                &sync.positions,
                sync.previous_positions.maybe(),
                &sync.names,
            )
                .par_join()
                .filter(|(e, p, _, _)| entity != *e && current_pos.within_distance(**p, 15))
                .collect();

            if !local.is_empty() {
                log::debug!("There are {} entities near {}", local.len(), named.name);
            }

            let by_id: AHashMap<Index, RemotePlayer<'_>> =
                local.into_iter().fold(AHashMap::new(), |mut hash, data| {
                    hash.insert(data.0.id(), data);
                    hash
                });

            let mut updates = visible
                .0
                .iter()
                .map(|idx| {
                    if let Some(remote_player) = by_id.get(idx) {
                        if let Some(previous) = remote_player.2 {
                            let direction = previous.0.direction_between(*remote_player.1);
                            let movement = if let Some(run_step) = previous.1 {
                                let run_direction = run_step.direction_between(previous.0);
                                if run_direction == Direction::None {
                                    Some(EntityMovement::Move {
                                        direction: direction as i32,
                                    })
                                } else {
                                    Some(EntityMovement::Run {
                                        directions: (run_direction as i32, direction as i32),
                                    })
                                }
                            } else if direction != Direction::None {
                                Some(EntityMovement::Move {
                                    direction: direction as i32,
                                })
                            } else {
                                None
                            };
                            PlayerUpdate::Update(movement, SyncBlocks::default())
                        } else {
                            PlayerUpdate::Update(None, SyncBlocks::default())
                        }
                    } else {
                        PlayerUpdate::Remove()
                    }
                })
                .collect::<Vec<PlayerUpdate>>();

            visible.0.retain(|idx| by_id.contains_key(&idx));

            let adds: Vec<Index> = by_id
                .values()
                .filter(|(entity, _, _, _)| !visible.0.contains(&entity.id()))
                .map(|remote_player| {
                    let mut blocks = SyncBlocks::default();

                    let mut equipment = Equipment::default();
                    equipment.hat = Some(Item { id: 1040 });
                    equipment.chest = Some(Item { id: 1121 });
                    equipment.legs = Some(Item { id: 1071 });

                    let block = Appearance {
                        name: (remote_player.3).name.to_string(),
                        gender: 0,
                        appearance_type: AppearanceType::Player(
                            equipment,
                            vec![0, 10, 18, 26, 33, 36, 42],
                        ),
                        combat_level: 69,
                        skill_level: 420,
                        colours: vec![0, 0, 0, 0, 0],
                    };
                    blocks.add_block(Box::new(block));
                    (
                        remote_player.0.id(),
                        PlayerUpdate::Add(
                            AddPlayer::new(
                                remote_player.0.id() as u16,
                                *current_pos,
                                *remote_player.1,
                            ),
                            blocks,
                        ),
                    )
                })
                .map(|(id, update)| {
                    updates.push(update);
                    id
                })
                .collect();
            adds.into_iter().for_each(|idx| {
                visible.0.insert(idx);
            });

            let update_region = match previous {
                Some(previous) => {
                    let previous = previous.0;
                    // Are we within region width in tiles * viewing distance
                    !previous.within_distance(*current_pos, 8 * 15)
                }
                None => true,
            };

            let has_moved = match previous {
                Some(previous) => !previous.0.eq(current_pos),
                None => false,
            };

            if update_region {
                net.send(
                    entity,
                    RegionChange {
                        position: *current_pos,
                    },
                );

                net.send(
                    entity,
                    PlayerSynchronization {
                        player_update: Some(PlayerUpdate::Update(
                            Some(EntityMovement::Teleport {
                                destination: *current_pos,
                                current: *current_pos,
                                changed_region: update_region,
                            }),
                            SyncBlocks::default(),
                        )),
                        other_players: updates,
                    },
                );
            } else if has_moved {
                let direction = previous.unwrap().0.direction_between(*current_pos);

                net.send(
                    entity,
                    PlayerSynchronization {
                        player_update: Some(PlayerUpdate::Update(
                            Some(EntityMovement::Move {
                                direction: direction as i32,
                            }),
                            SyncBlocks::default(),
                        )),
                        other_players: updates,
                    },
                );
            } else {
                net.send(
                    entity,
                    PlayerSynchronization {
                        player_update: None,
                        other_players: updates,
                    },
                );
            }

            net.send(entity, NpcSynchronization);
        }
    }
}

#[derive(Default)]
pub struct EntityPathfindingSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, EntityPathfindingSystem> for EntityPathfindingSystemDesc {
    fn build(self, world: &mut World) -> EntityPathfindingSystem {
        <EntityPathfindingSystem as System<'_>>::SystemData::setup(world);
        let reader = world.fetch_mut::<PacketEventChannel>().register_reader();
        EntityPathfindingSystem::new(reader)
    }
}

pub struct EntityPathfindingSystem {
    reader: ReaderId<EntityPacketEvent>,
}

impl EntityPathfindingSystem {
    pub fn new(reader: ReaderId<EntityPacketEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for EntityPathfindingSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, PacketEventChannel>,
        ReadExpect<'a, CollisionDetector>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, PreviousPosition>,
        WriteStorage<'a, Pathfinder>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (entities, packets, detector, mut pos_storage, mut previous_pos, mut path_storage, lazy): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("pathfinding");

        for (player, packet) in packets.read(&mut self.reader) {
            if let PacketEvent::Gameplay(packet) = packet {
                if let GameplayEvent::Walk(packet) = packet {
                    let pathfinder = match path_storage.get_mut(*player) {
                        Some(pathfinder) => pathfinder,
                        None => continue,
                    };

                    let current = pos_storage.get(*player).unwrap();

                    pathfinder.set_running(packet.running);
                    pathfinder.walk_path(&detector, *current, packet.path.clone());
                }
            }
        }

        (
            &entities,
            &mut pos_storage,
            (&mut previous_pos).maybe(),
            &mut path_storage,
        )
            .par_join()
            .for_each(|(entity, current, previous, pathfinder)| {
                if let Some(next_step) = pathfinder.next_step() {
                    // Confusing as it might be, next step is actually the run step
                    let run_step = if pathfinder.is_running() {
                        pathfinder.next_step()
                    } else {
                        None
                    };

                    if run_step.is_some() {
                        if let Some(previous) = previous {
                            previous.0 = next_step;
                            previous.1 = Some(*current);
                        } else {
                            lazy.insert(entity, PreviousPosition(next_step, Some(*current)));
                        }
                    } else {
                        if let Some(previous) = previous {
                            previous.0 = *current;
                            previous.1 = None;
                        } else {
                            lazy.insert(entity, PreviousPosition(*current, None));
                        }
                    }

                    *current = run_step.unwrap_or(next_step);
                } else if let Some(previous) = previous {
                    previous.0 = current.clone();
                    previous.1 = None;
                }
            });
    }

    fn running_time(&self) -> RunningTime {
        RunningTime::VeryLong
    }
}
