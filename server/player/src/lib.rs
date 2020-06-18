use amethyst::{
    core::bundle::SystemBundle,
    ecs::{prelude::*, DispatcherBuilder, Entities, Read, ReadStorage, System, World, Write},
    Result,
};

use mithril_server_net::MithrilTransportResource;
use mithril_server_types::{NewPlayer, ConnectionIsaac};
use mithril_core::net::packets::{IdAssignment, SwitchTabInterface, UpdateSkill};

pub struct PlayerEntityBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PlayerEntityBundle {
    fn build(self, world: &mut World, dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        // TODO: add systems rewritten for amethyst compatibility to the dispatcher
        <SendInitialPackets as System<'_>>::SystemData::setup(world);
        dispatcher.add(SendInitialPackets, "send_join_packets", &[]);
        Ok(())
    }
}

pub struct SendInitialPackets;

impl<'a> System<'a> for SendInitialPackets {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Write<'a, MithrilTransportResource>,
        ReadStorage<'a, NewPlayer>,
        ReadStorage<'a, ConnectionIsaac>
    );

    fn run(&mut self, (entities, lazy, mut transport, isaac, new_player): Self::SystemData) {
        for (entity, _, _) in (&entities, &isaac, &new_player).join() {
            lazy.remove::<NewPlayer>(entity);

            transport.send(entity, IdAssignment {
                is_member: true,
                entity_id: 1,
            });

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
                transport.send(entity, SwitchTabInterface {
                    interface_id: open_tabs[i],
                    tab_id: i as u8
                });
            }

            for i in 0..25 {
                transport.send(entity, UpdateSkill {
                    skill_id: i,
                    experience: 0,
                    level: 1,
                });
            }
        }
    }
}
