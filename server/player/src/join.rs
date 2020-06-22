use amethyst::{
    core::{Named, SystemDesc},
    ecs::prelude::*,
};

use mithril_core::{
    net::packets::{IdAssignment, ServerMessage, SwitchTabInterface, UpdateSkill},
    pos::Position,
};
use mithril_server_net::MithrilTransportResource;
use mithril_server_types::{NewPlayer, Pathfinder, VisiblePlayers};

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
    );

    fn run(&mut self, (entities, lazy, mut transport, new_player, named): Self::SystemData) {
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

            for i in 0..open_tabs.len() {
                transport.send(
                    player,
                    SwitchTabInterface {
                        interface_id: open_tabs[i],
                        tab_id: i as u8,
                    }
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
                    ClearRegion {
                        player: Position::default(),
                        region: Position::default(),
                    },
                );

                transport.send(
                    player,
                    RegionChange {
                        position: Position::default(),
                    },
                );

                let mut equipment = Equipment::default();
                equipment.hat = Some(Item { id: 1042 });
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
                blocks.add_block(Box::new(appearance));

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

            lazy.insert(player, Position::default());
            lazy.insert(player, Pathfinder::default());
            lazy.insert(player, VisiblePlayers::default());
        }
    }
}
