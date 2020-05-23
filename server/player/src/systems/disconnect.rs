use specs::prelude::*;
use mithril_server_types::{Network, WorkerToServerMessage, Name};

pub struct DisconnectClients;

impl<'a> System<'a> for DisconnectClients {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Network>,
    );

    fn run(&mut self, (entities, name_storage, networked_storage): Self::SystemData) {
        (&*entities, &name_storage, &networked_storage).par_join().for_each(|(entity, name, networked)| {
            while let Ok(msg) = networked.rx.lock().try_recv() {
                match msg {
                    WorkerToServerMessage::Disconnect { reason } => {
                        log::debug!("Disconnecting {}: {}", name, reason);
                        let _ = entities.delete(entity);
                    }
                }
            }
        });
    }
}