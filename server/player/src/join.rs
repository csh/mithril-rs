use amethyst::{
    core::{Named, SystemDesc},
    ecs::prelude::*,
};
use hibitset::BitSet;
use ahash::AHashSet;

use mithril_core::{
    net::packets::{IdAssignment, ServerMessage, SwitchTabInterface, UpdateSkill},
    pos::Position,
};
use mithril_server_net::MithrilTransportResource;
use mithril_server_types::{NewPlayer, Pathfinder, VisiblePlayers, VisibleRegions, VisibleObjects, WorldObjectData, Deleted};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Default)]
pub struct SendInitialPacketsSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, SendInitialPackets> for SendInitialPacketsSystemDesc {
    fn build(self, world: &mut World) -> SendInitialPackets {
        <SendInitialPackets as System<'_>>::SystemData::setup(world);
        SendInitialPackets
    }
}

pub struct SendInitialPackets;

impl<'a> System<'a> for SendInitialPackets {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Write<'a, MithrilTransportResource>,
        ReadStorage<'a, NewPlayer>,
        ReadStorage<'a, Named>,
        ReadStorage<'a, WorldObjectData>,
        ReadStorage<'a, Deleted>,
    );

    fn run(&mut self, (entities, lazy, mut transport, new_player, named, object_data, deleted): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("player join");

        for (player, named, _) in (&entities, &named, &new_player).join() {
            lazy.remove::<NewPlayer>(player);

            transport.send(
                player,
                IdAssignment {
                    is_member: true,
                    entity_id: 1,
                },
            );

            let open_tabs: [u16; 14] = [
                2423,
                3917,
                638,
                3213,
                1644,
                5608,
                1151,
                u16::MAX,
                5065,
                5715,
                2449,
                904,
                147,
                962,
            ];

            for (i, interface_id) in open_tabs.iter().enumerate() {
                transport.send(
                    player,
                    SwitchTabInterface {
                        interface_id: *interface_id,
                        tab_id: i as u8,
                    },
                );
            }

            for i in 0..25 {
                transport.send(
                    player,
                    UpdateSkill {
                        skill_id: i,
                        experience: 0,
                        level: 1,
                    },
                );
            }

            transport.send(
                player,
                ServerMessage {
                    message: "Mithril:tradereq:".to_owned(),
                },
            );

            {
                use mithril_core::net::packets::*;

                transport.send(
                    player,
                    ClearRegion::new(Position::default(), (&Position::default()).into()),
                );

                transport.send(
                    player,
                    RegionChange {
                        position: Position::default(),
                    },
                );

                let mut equipment = Equipment::default();
                equipment.hat = Some(Item { id: 1040 });
                equipment.chest = Some(Item { id: 1121 });
                equipment.legs = Some(Item { id: 1071 });

                let appearance = Appearance {
                    name: named.name.to_string(),
                    gender: 0,
                    appearance_type: AppearanceType::Player(
                        equipment,
                        vec![0, 10, 18, 26, 33, 36, 42],
                    ),
                    combat_level: 69,
                    skill_level: 420,
                    colours: vec![0, 0, 0, 0, 0],
                };

                let mut blocks = SyncBlocks::default();
                blocks.add_block(appearance.into());

                transport.send(
                    player,
                    PlayerSynchronization {
                        player_update: Some(PlayerUpdate::Update(
                            Some(EntityMovement::Teleport {
                                changed_region: true,
                                current: Position::default(),
                                destination: Position::default(),
                            }),
                            blocks,
                        )),
                        other_players: vec![],
                    },
                )
            }

            let bitset = (&entities, &object_data, !&deleted)
                .join()
                .fold(BitSet::new(), |mut bitset, (entity, ..)| {
                    bitset.add(entity.id());
                    bitset    
                });

            lazy.insert(player, Position::default());
            lazy.insert(player, Pathfinder::default());
            lazy.insert(player, VisiblePlayers::default());
            lazy.insert(player, VisibleRegions(AHashSet::new()));
            lazy.insert(player, VisibleObjects(bitset));
        }
    }
}
