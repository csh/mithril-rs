use legion::command::CommandBuffer;
use parking_lot::Mutex;

use mithril_server_net::NewClient;
use mithril_server_types::{Named, Networked};

mod connect;

pub use connect::*;

pub fn create(commands: &mut CommandBuffer, info: NewClient) {
    let entity = info.entity;
    commands.add_component(entity, Named(info.username));
    commands.add_component(entity, info.ip);
    commands.add_component(
        entity,
        Networked {
            rx: Mutex::new(info.rx),
            tx: info.tx,
        },
    );
}
