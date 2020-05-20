use legion::prelude::*;

use mithril_server_net::{
    ListenerToServerMessage, NetworkManager, NewClient, ServerToListenerMessage,
};
use mithril_server_packets::Packets;
use mithril_server_types::{
    AuthenticationResult, Named, Networked, ServerToWorkerMessage, WorkerToServerMessage,
};

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
                            packet: Box::new(mithril_core::net::packets::IdAssignment {
                                is_member: true,
                                entity_id: 1
                            }),
                        });

                        crate::create(commands, new_client);
                    }
                }
            }
        })
}
