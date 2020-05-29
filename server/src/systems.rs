use specs::prelude::*;

use mithril_server_player as player;

pub fn build_dispatcher<'a, 'b>() -> Dispatcher<'a, 'b> {
    DispatcherBuilder::new()
        .with(player::systems::DisconnectClients, "disconnect", &[])
        .with(player::systems::PollNewClients, "poll_clients", &[])
        .with(player::systems::EntityPathfinding, "pathfinder", &[])
        .with(player::systems::PlayerSync, "player_sync", &["poll_clients", "pathfinder"])
        .with(player::systems::ResetPreviousPosition, "reset_previous", &["player_sync"])
        .build()
}
