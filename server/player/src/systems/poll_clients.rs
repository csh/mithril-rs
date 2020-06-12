#![allow(clippy::type_complexity)]
use mithril_core::net::packets::{IdAssignment, SwitchTabInterface};
use mithril_core::pos::Position;
use mithril_server_net::{ListenerToServerMessage, NetworkManager, ServerToListenerMessage};
use mithril_server_types::{
    AuthenticationResult, Name, Network, Pathfinder, ServerToWorkerMessage,
};
use parking_lot::Mutex;
use specs::prelude::*;

pub struct PollNewClients;

impl<'a> System<'a> for PollNewClients {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, NetworkManager>,
        WriteStorage<'a, Name>,
        WriteStorage<'a, Network>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Pathfinder>,
    );

    fn run(
        &mut self,
        (
            entities,
            network_manager,
            mut named_storage,
            mut network_storage,
            mut pos_storage,
            mut queue_storage,
        ): Self::SystemData,
    ) {
        while let Ok(msg) = network_manager.rx.lock().try_recv() {
            match msg {
                ListenerToServerMessage::CreateEntity => {
                    let entity = entities.create();
                    let _ = network_manager
                        .tx
                        .send(ServerToListenerMessage::EntityCreated(entity));
                }
                ListenerToServerMessage::DestroyEntity(entity) => {
                    entities.delete(entity).unwrap();
                }
                ListenerToServerMessage::Authenticate(new_client) => {
                    let _username = new_client.username.clone();
                    let _password = new_client.password.clone();
                    log::debug!("Attempting to authenticate {}", _username);
                    let _ = new_client.tx.send(ServerToWorkerMessage::Authenticated(
                        AuthenticationResult::Success,
                    ));

                    let network = Network {
                        ip: new_client.ip,
                        tx: new_client.tx,
                        rx: Mutex::new(new_client.rx),
                    };

                    network.send(IdAssignment {
                        is_member: true,
                        entity_id: 1,
                    });

                    // TODO: Move this definition elsewhere. For now, enjoy the fact the client displays more than "Connection lost" :)
                    let tabs: [u16; 14] = [
                        2423,
                        3917,
                        638,
                        3213,
                        1644,
                        5608,
                        1151,
                        std::u16::MAX,
                        5065,
                        5715,
                        2449,
                        904,
                        147,
                        962,
                    ];
                    for (tab_id, interface_id) in tabs.iter().enumerate() {
                        network.send(SwitchTabInterface {
                            interface_id: *interface_id,
                            tab_id: tab_id as u8,
                        });
                    }

                    for i in 0..25 {
                        network.send(mithril_core::net::packets::UpdateSkill {
                            skill_id: i,
                            experience: 0,
                            level: 1,
                        });
                    }

                    network.send(mithril_core::net::packets::ServerMessage {
                        message: String::from("Mithril:tradereq:"),
                    });

                    named_storage
                        .insert(new_client.entity, Name(_username))
                        .unwrap();
                    network_storage.insert(new_client.entity, network).unwrap();
                    pos_storage
                        .insert(new_client.entity, Position::default())
                        .unwrap();
                    queue_storage
                        .insert(new_client.entity, Pathfinder::default())
                        .unwrap();
                }
            }
        }
    }
}
