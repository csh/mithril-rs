use mithril_core::net::packets::{EntityMovement, NpcSynchronization, PlayerSynchronization, Walk};
use mithril_core::net::PacketType;
use mithril_core::pos::Position;
use mithril_server_packets::Packets;
use mithril_server_types::{CollisionDetector, Name, Network, Pathfinder, PreviousPosition};
use specs::prelude::*;
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
    type SystemData = (Entities<'a>, PlayerSyncStorage<'a>);

    fn run(&mut self, (entities, sync): Self::SystemData) {
        (
            &entities,
            &sync.names,
            &sync.network,
            &sync.positions,
            sync.previous_positions.maybe(),
        )
            .par_join()
            .for_each(|(entity, name, network, current_pos, previous)| {
                let (local, local_new): (
                    Vec<(Entity, &Position, Option<&PreviousPosition>)>,
                    Vec<(Entity, &Position, Option<&PreviousPosition>)>,
                ) = (&entities, &sync.positions, sync.previous_positions.maybe())
                    .par_join()
                    .filter(|(e, p, _)| entity != *e && current_pos.within_distance(**p, 15))
                    .partition(|(_, _, previous)| previous.is_some());

                if !local.is_empty() {
                    log::debug!("There are {} entities near {}", local.len(), name);
                }

                if !local_new.is_empty() {
                    log::debug!("Spawning {} entities near {}", local_new.len(), name);
                }

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
                        player_update: Some(EntityMovement::Teleport {
                            destination: *current_pos,
                            current: *current_pos,
                            changed_region: update_region,
                        }),
                    });
                } else if has_moved {
                    let direction = previous.unwrap().0.direction_between(*current_pos);
                    network.send(PlayerSynchronization {
                        player_update: Some(EntityMovement::Move {
                            direction: direction as i32,
                        }),
                    });
                } else {
                    network.send(PlayerSynchronization {
                        player_update: None,
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
