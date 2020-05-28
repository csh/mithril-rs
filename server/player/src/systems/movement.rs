use mithril_core::pos::Position;
use mithril_server_types::{Name, Network, PreviousPosition};
use specs::prelude::*;
use mithril_core::net::packets::{PlayerSynchronization, EntityMovement, NpcSynchronization};

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
        PlayerSyncStorage<'a>
    );

    fn run(
        &mut self,
        (entities, sync): Self::SystemData,
    ) {
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

                let region_changed = match previous {
                    Some(previous) => {
                        let previous = previous.0;
                        !(previous.get_region_x() == current_pos.get_region_x() && previous.get_region_y() == current_pos.get_region_y())
                    }
                    None => true
                };

                let has_moved = match previous {
                    Some(previous) => {
                        !previous.eq(current_pos)
                    }
                    None => false
                };

                if region_changed {
                    log::debug!("{} has moved", name);

                    network.send(mithril_core::net::packets::RegionChange {
                        position: *current_pos,
                    });

                    network.send(PlayerSynchronization {
                        player_update: Some(EntityMovement::Teleport {
                            destination: *current_pos,
                            current: *current_pos,
                            changed_region: region_changed
                        })
                    });
                } else if has_moved {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let direction = rng.gen_range(0, 7);
                    network.send(PlayerSynchronization {
                        player_update: Some(EntityMovement::Move {
                            direction
                        })
                    });
                } else {
                    network.send(PlayerSynchronization {
                        player_update: None
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
        Read<'a, LazyUpdate>
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
