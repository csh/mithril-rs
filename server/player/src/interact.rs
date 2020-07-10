use amethyst::{
    core::{bundle::SystemBundle, Named, SystemDesc},
    ecs::{
        Builder, Entities, Entity, LazyUpdate, Read, ReadExpect, ReadStorage, System,
        SystemData, World, Write, WriteStorage,
        Join,
        WorldExt,
    },
    network::simulation::{NetworkSimulationEvent, TransportResource},
    shrev::{EventChannel, ReaderId},
    Result,
};

use mithril_core::{
    pos::{Position, Direction},
    net::packets::{
        GameplayEvent,
        PacketEvent,
        ForceChat,
        Animation,
        ObjectType,
    },

};
use mithril_server_types::{Deleted, WorldObjectData, ObjectDefinitions, Updates};
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
        WriteStorage<'a, Updates>,
    );

    fn run(&mut self, (entities, channel, net, definitions, lazy, position, object_data, mut deleted, mut updates): Self::SystemData) { 
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

                let entity = (&entities, &object_data, &position, !&deleted)
                    .join()
                    .filter(|(_, _, position, _)| position.x as u16 == action.x && position.y as u16 == action.y)
                    .filter(|(_, data, _, _)| {
                        match data {
                            WorldObjectData::Object {
                                id,
                                ..
                            } if *id == action.object_id => true,
                            _ => false
                        }
                    })
                    .map(|(entity, data, pos, _)| (entity, pos, data))
                    .nth(0);

                let (entity, pos, data) = match entity {
                    Some(entity) => entity,
                    None => continue
                };

                if action_str == "Open" && def.name == "Door" {
                    updates.get_mut(*player).unwrap().0.push(ForceChat {
                        message: "BOOM!!!".to_owned(),
                    }.into());
                    updates.get_mut(*player).unwrap().0.push(Animation {
                        id: 422,
                        delay: 0,
                    }.into()); 

                    deleted.insert(entity, Deleted).expect("Oof");
                }
            }
        }
    }
}

