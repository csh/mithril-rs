use legion::prelude::*;

use mithril_server_net::{
    ListenerToServerMessage, NetworkManager, NewClient, ServerToListenerMessage,
};
use mithril_server_packets::Packets;
use mithril_server_types::{
    AuthenticationResult, Named, Networked, ServerToWorkerMessage, WorkerToServerMessage,
};

use mithril_core::net::packets::{IdAssignment, SwitchTabInterface};

pub fn poll_disconnect() -> Box<dyn Schedulable> {
    SystemBuilder::new("poll_disconnect")
        .with_query(<(Read<Networked>, Read<Named>)>::query())
        .build(move |commands, world, _, queries| {
            for (entity, (networked, named)) in queries.iter_entities(&mut *world) {
                while let Ok(msg) = networked.rx.lock().try_recv() {
                    match msg {
                        WorkerToServerMessage::Disconnect => {
                            log::debug!("Disconnecting {}", named.0);
                            commands.delete(entity);
                        }
                    }
                }
            }
        })
}

pub fn poll_new_clients() -> Box<dyn Schedulable> {
    SystemBuilder::new("poll_new_clients")
        .read_resource::<NetworkManager>()
        .build(|commands, _, network_manager, _| {
            while let Ok(msg) = network_manager.rx.lock().try_recv() {
                match msg {
                    ListenerToServerMessage::CreateEntity => {
                        let entity = commands.start_entity().build();
                        let _ = network_manager
                            .tx
                            .send(ServerToListenerMessage::EntityCreated(entity));
                    }
                    ListenerToServerMessage::DestroyEntity(entity) => {
                        commands.delete(entity);
                    }
                    ListenerToServerMessage::Authenticate(new_client) => {
                        let _username = new_client.username.clone();
                        let _password = new_client.password.clone();
                        log::debug!("Attempting to authenticate {}", _username);
                        let _ = new_client.tx.send(ServerToWorkerMessage::Authenticated(
                            AuthenticationResult::Success,
                        ));

                        let _ = new_client.tx.send(ServerToWorkerMessage::Dispatch {
                            packet: Box::new(IdAssignment {
                                is_member: true,
                                entity_id: 1
                            }),
                        });

                        // TODO: Move this definition elsewhere. For now, enjoy the fact the client displays more than "Connection lost" :)
                        let tabs: [u16; 14] = [2423, 3917, 638, 3213, 1644, 5608, 1151, u16::MAX, 5065, 5715, 2449, 904, 147, 962];
                        for i in 0..tabs.len() {
                            let _ = new_client.tx.send(ServerToWorkerMessage::Dispatch {
                                packet: Box::new(SwitchTabInterface {
                                    interface_id: tabs[i],
                                    tab_id: i as u8
                                })
                            });
                        }

                        crate::create(commands, new_client);
                    }
                }
            }
        })
}
