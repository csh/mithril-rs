use specs::prelude::*;

use mithril_server_player as player;

pub fn build_dispatcher<'a, 'b>() -> Dispatcher<'a, 'b> {
    DispatcherBuilder::new()
        .with(player::systems::DisconnectClients, "disconnect", &[])
        .with(player::systems::PollNewClients, "poll_clients", &[])
        .build()
}
