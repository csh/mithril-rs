#![allow(clippy::type_complexity)]
use mithril_core::net::packets::{EntityMovement, NpcSynchronization, PlayerSynchronization, Walk, PlayerUpdate, SyncBlocks, AddPlayer};
use mithril_core::net::PacketType;
use mithril_core::pos::Position;
use mithril_server_packets::Packets;
use mithril_server_types::{CollisionDetector, Name, Network, Pathfinder, PreviousPosition, VisiblePlayers};
use specs::prelude::*;
use specs::world::Index;
use ahash::AHashMap;
use std::sync::Arc;

#[derive(SystemData)]
pub struct PlayerSyncStorage<'a> {
    names: ReadStorage<'a, Name>,
    network: ReadStorage<'a, Network>,
    positions: ReadStorage<'a, Position>,
    previous_positions: ReadStorage<'a, PreviousPosition>,
    visible_players: WriteStorage<'a, VisiblePlayers>,
}

pub struct PlayerSync;

impl<'a> System<'a> for PlayerSync {
    type SystemData = (Entities<'a>, PlayerSyncStorage<'a>);

    #[allow(clippy::type_complexity)]
    fn run(&mut self, (entities, mut sync): Self::SystemData) {
        (
            &entities,
            &sync.names,
            &sync.network,
            &sync.positions,
            sync.previous_positions.maybe(),
            &sync.visible_players,
        )
            .par_join()
            .for_each(|(entity, name, network, current_pos, previous, mut visible)| {
                type RemotePlayer<'a> = (Entity, &'a Position, Option<&'a PreviousPosition>);
                let local: Vec<RemotePlayer<'a>> 
                    = (&entities, &sync.positions, sync.previous_positions.maybe())
                    .par_join()
                    .filter(|(e, p, _)| entity != *e && current_pos.within_distance(**p, 15))
                    .collect();

                if !local.is_empty() {
                    log::debug!("There are {} entities near {}", local.len(), name);
                }

                let new_visible = visible.0;
                let mut by_id: AHashMap<Index, RemotePlayer<'a>> = local.into_iter().fold(AHashMap::new(), |mut hash, data| {
                    hash.insert(data.0.id(), data);
                    hash
                });

                let mut updates = visible.0.iter().map(|idx| {
                    if let Some(remote_player) = by_id.get(idx) {
                        // TODO: support running
                        // TODO: add direction
                        PlayerUpdate::Update(Some(
                            EntityMovement::Move {
                                direction: 0,
                            }
                        ), SyncBlocks::default())
                    } else {
                        PlayerUpdate::Remove()
                    }
                }).collect::<Vec<PlayerUpdate>>();
                
                let new_ids = by_id.values()
                    .filter(|(entity, _, _)| !visible.0.contains(&entity.id()))
                    .map(|remote_player| {
                        (remote_player.0.id(), PlayerUpdate::Add(AddPlayer {
                            id: remote_player.0.id() as u16,
                            dx: 0,
                            dy: 0,
                        }, SyncBlocks::default()))   
                    })
                    .map(|(id, update)| {
                        updates.push(update);
                        id
                    }).collect::<Vec<Index>>();
                // Convert to nice call
                new_visible.append(new_ids);

                let update_region = match previous {
                    Some(previous) => {
                        let previous = previous.0;
                        // Are we within region width in tiles * viewing distance
                        !previous.within_distance(*current_pos, 8 * 15)
                    }
                    None => true,
                };

                let has_moved = match previous {
                    Some(previous) => !previous.eq(current_pos),
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
                        other_players: vec![]
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
                        other_players: vec![],
                    });
                } else {
                    network.send(PlayerSynchronization {
                        player_update: None,
                        other_players: vec![],
                    });
                }

                network.send(NpcSynchronization);
            });
    }
}

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
}

pub struct EntityPathfinding;

impl<'a> System<'a> for EntityPathfinding {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Arc<Packets>>,
        ReadExpect<'a, CollisionDetector>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Pathfinder>,
    );

    fn run(
        &mut self,
        (entities, packets, detector, mut pos_storage, mut path_storage): Self::SystemData,
    ) {
        (&entities, &mut pos_storage, &mut path_storage)
            .par_join()
            .for_each(|(entity, current, pathfinder)| {
                let walk_packet = packets
                    .received_from::<Walk>(entity, PacketType::Walk)
                    .last();
                if let Some(walk_packet) = walk_packet {
                    pathfinder.set_running(walk_packet.running);
                    pathfinder.walk_path(&detector, *current, walk_packet.path);
                }

                if let Some(next_step) = pathfinder.next_step() {
                    *current = next_step
                }
            });
    }
}
