use amethyst::{
    core::{bundle::SystemBundle, Named, SystemDesc},
    ecs::{
        DispatcherBuilder, Entities, Entity, LazyUpdate, Read, ReadExpect, ReadStorage, System,
        SystemData, World, Write, WriteStorage,
        Join,
    },
    network::simulation::{NetworkSimulationEvent, TransportResource},
    shrev::{EventChannel, ReaderId},
    Result,
};

use mithril_core::{
    pos::Position,
    net::packets::{
        GameplayEvent,
        PacketEvent
    },

};
use mithril_server_types::{Deleted, WorldObjectData, ObjectDefinitions};
use mithril_server_net::{PacketEventChannel, EntityPacketEvent, MithrilTransportResource};

pub struct InteractSystem {
    pub reader: ReaderId<EntityPacketEvent>,
}

impl<'a> System<'a> for InteractSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, PacketEventChannel>,
        Write<'a, MithrilTransportResource>,
        ReadExpect<'a, ObjectDefinitions>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, WorldObjectData>,
        WriteStorage<'a, Deleted>,
    );

    fn run(&mut self, (entities, channel, net, definitions, lazy, position, object_data, mut deleted): Self::SystemData) { 
        for (player, event) in channel.read(&mut self.reader) {
            if let PacketEvent::Gameplay(gameplay_event) = event {
                let action = match gameplay_event {
                    GameplayEvent::FirstObjectAction(action) => action,
                    GameplayEvent::SecondObjectAction(action) => action,
                    GameplayEvent::ThirdObjectAction(action) => action,
                    _ => continue
                };

                let def = match definitions.get(action.object_id) {
                    Some(def) => def,
                    None => continue
                };

                let action_str = match &def.interact_actions[action.action_index as usize] {
                    Some(action) => action,
                    None => continue    
                };

                let entity = (&entities, &object_data, &position)
                    .join()
                    .filter(|(_, _, position)| position.x as u16 == action.x && position.y as u16 == action.y)
                    .filter(|(_, data, _)| {
                        match data {
                            WorldObjectData::Object {
                                id,
                                ..
                            } if *id == action.object_id => true,
                            _ => false
                        }
                    })
                    .map(|(entity, _, pos)| (entity, pos))
                    .nth(0);

                let (entity, pos) = match entity {
                    Some(entity) => entity,
                    None => continue
                };

                if action_str == "Open" && def.name == "Door" {
                    println!("Opening door");
                    dbg!(pos);
                    deleted.insert(entity, Deleted).expect("Oof");
                }
            }
        }
    }
}

