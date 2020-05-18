use legion::prelude::*;
use parking_lot::Mutex;

use mithril_server_player as player;

pub fn build_executor() -> Executor {
    Executor::new(vec![player::poll_disconnect(), player::poll_new_clients()])
}
