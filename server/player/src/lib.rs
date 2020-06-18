use amethyst::{
    core::{bundle::SystemBundle},
    ecs::{World, DispatcherBuilder},
    Result
};

pub struct PlayerEntityBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PlayerEntityBundle {
    fn build(self, _world: &mut World, _dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        // TODO: add systems rewritten for amethyst compatibility to the dispatcher
        Ok(())
    }
}