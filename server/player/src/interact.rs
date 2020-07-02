use amethyst::{
    core::{bundle::SystemBundle, Named, SystemDesc},
    ecs::{
        DispatcherBuilder, Entities, Entity, LazyUpdate, Read, ReadExpect, ReadStorage, System,
        SystemData, World, Write, WriteStorage,
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
use mithril_server_types::{Deleted, WorldObjectData};
use mithril_server_net::{PacketEventChannel, EntityPacketEvent, MithrilTransportResource};

pub struct InteractSystem {
    pub reader: ReaderId<EntityPacketEvent>,
}

impl<'a> System<'a> for InteractSystem {
    type SystemData = (
        Read<'a, PacketEventChannel>,
        Write<'a, MithrilTransportResource>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, WorldObjectData>,
        WriteStorage<'a, Deleted>,
    );

    fn run(&mut self, (channel, mut net, lazy, position, object_data, deleted): Self::SystemData) { 
        for (player, event) in channel.read(&mut self.reader) {
            if let PacketEvent::Gameplay(gameplay_event) = event {
                let action = match gameplay_event {
                    GameplayEvent::FirstObjectAction(action) => action,
                    GameplayEvent::SecondObjectAction(action) => action,
                    GameplayEvent::ThirdObjectAction(action) => action,
                    _ => continue
                };
                println!("Hello interact");
                dbg!(action);
            }
        }
    }
}

