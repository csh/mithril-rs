use amethyst::{
    core::{bundle::SystemBundle, SystemDesc},
    ecs::prelude::*,
    Result,
};

mod join;
mod movement;
mod objects;

pub struct PlayerEntityBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PlayerEntityBundle {
    fn build(self, world: &mut World, dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        dispatcher.add(
            join::SendInitialPacketsSystemDesc::default().build(world),
            "send_join_packets",
            &[],
        );

        dispatcher.add(
            movement::EntityPathfindingSystemDesc::default().build(world),
            "entity_pathfinding",
            &[],
        );

        dispatcher.add(
            movement::PlayerSyncSystemDesc::default().build(world),
            "player_sync",
            &["entity_pathfinding"],
        );

        dispatcher.add(
            objects::RegionUpdateSystemDesc::default().build(world),
            "object_sync",
            &["entity_pathfinding"],
        );

        Ok(())
    }
}
