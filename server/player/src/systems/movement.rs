#![allow(clippy::type_complexity)]
use ahash::AHashMap;
use mithril_core::net::packets::{
    AddPlayer, EntityMovement, NpcSynchronization, PlayerSynchronization, PlayerUpdate, SyncBlocks,
    Walk, Appearance, AppearanceType, Equipment, Item
};
use mithril_core::net::PacketType;
use mithril_core::pos::{Position, Direction};
use mithril_server_packets::Packets;
use mithril_server_types::{
    CollisionDetector, Name, Network, Pathfinder, PreviousPosition, VisiblePlayers,
};
use specs::prelude::*;
use specs::world::Index;
use std::sync::Arc;

#[derive(SystemData)]
pub struct PlayerSyncStorage<'a> {
    names: ReadStorage<'a, Name>,
    network: ReadStorage<'a, Network>,
    positions: ReadStorage<'a, Position>,
    previous_positions: ReadStorage<'a, PreviousPosition>,
}

pub struct PlayerSync;

impl<'a> System<'a> for PlayerSync {
    type SystemData = (
        Entities<'a>,
        PlayerSyncStorage<'a>,
        WriteStorage<'a, VisiblePlayers>,
    );

    #[allow(clippy::type_complexity)]
    fn run(&mut self, (entities, sync, mut visible_players): Self::SystemData) {
        (
            &entities,
            &sync.names,
            &sync.network,
            &sync.positions,
            sync.previous_positions.maybe(),
            &mut visible_players,
        )
            .par_join()
            .for_each(
                |(entity, name, network, current_pos, previous, visible)| {
                    type RemotePlayer<'a> = (Entity, &'a Position, Option<&'a PreviousPosition>, &'a Name);
                    let local: Vec<RemotePlayer<'a>> =
                        (&entities, &sync.positions, sync.previous_positions.maybe(), &sync.names)
                            .par_join()
                            .filter(|(e, p, _, _)| {
                                entity != *e && current_pos.within_distance(**p, 15)
                            })
                            .collect();

                    if !local.is_empty() {
                        log::debug!("There are {} entities near {}", local.len(), name);
                    }

                    let by_id: AHashMap<Index, RemotePlayer<'a>> =
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
                                                direction: direction as i32
                                            })
                                        } else {
                                            Some(EntityMovement::Run {
                                                directions: (
                                                    run_direction as i32,
                                                    direction as i32,
                                                )
                                            })
                                        }
                                    } else if direction != Direction::None {
                                        Some(EntityMovement::Move {
                                            direction: direction as i32
                                        })
                                    } else {
                                        None    
                                    };
                                    PlayerUpdate::Update(
                                        movement,
                                        SyncBlocks::default(),
                                    )
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
                            equipment.hat = Some(Item {id: 1040});
                            equipment.chest = Some(Item{ id: 1121});
                            equipment.legs = Some(Item {id: 1071});
                            
                            let block = Appearance {
                                name: (remote_player.3).0.to_owned(),
                                gender: 0,
                                appearance_type: AppearanceType::Player(equipment, vec![0, 10, 18, 26, 33, 36, 42]),
                                combat_level: 69,
                                skill_level: 420,
                                colours: vec![0,0,0,0,0]
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
                        network.send(mithril_core::net::packets::RegionChange {
                            position: *current_pos,
                        });

                        network.send(PlayerSynchronization {
                            player_update: Some(PlayerUpdate::Update(
                                Some(EntityMovement::Teleport {
                                    destination: *current_pos,
                                    current: *current_pos,
                                    changed_region: update_region,
                                }),
                                SyncBlocks::default(),
                            )),
                            other_players: updates,
                        });
                    } else if has_moved {
                        let direction = previous.unwrap().0.direction_between(*current_pos);
                        
                        network.send(PlayerSynchronization {
                            player_update: Some(PlayerUpdate::Update(
                                Some(EntityMovement::Move {
                                    direction: direction as i32,
                                }),
                                SyncBlocks::default(),
                            )),
                            other_players: updates,
                        });
                    } else {
                        network.send(PlayerSynchronization {
                            player_update: None,
                            other_players: updates,
                        });
                    }

                    network.send(NpcSynchronization);
                },
            );
    }
}
/*
pub struct ResetPreviousPosition;

impl<'a> System<'a> for ResetPreviousPosition {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, PreviousPosition>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (entities, pos_storage, mut prev_storage, lazy): Self::SystemData) {
        (&pos_storage, &mut prev_storage)
            .par_join()
            .for_each(|(current, mut previous)| {
                previous.0 = *current;
            });

        (&entities, &pos_storage, !&prev_storage)
            .par_join()
            .for_each(|(entity, current, _)| {
                lazy.insert(entity, PreviousPosition(*current));
            });
    }
}*/

pub struct EntityPathfinding;

impl<'a> System<'a> for EntityPathfinding {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Arc<Packets>>,
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
        (&entities, &mut pos_storage, (&mut previous_pos).maybe(), &mut path_storage)
            .par_join()
            .for_each(|(entity, current, previous, pathfinder)| {
                let walk_packet = packets
                    .received_from::<Walk>(entity, PacketType::Walk)
                    .last();
                if let Some(walk_packet) = walk_packet {
                    pathfinder.set_running(walk_packet.running);
                    pathfinder.walk_path(&detector, *current, walk_packet.path);
                }
                
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
}
