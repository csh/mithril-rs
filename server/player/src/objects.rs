use amethyst::{
    core::SystemDesc,
    ecs::prelude::*
};

use mithril_core::pos::Position;
use mithril_server_net::MithrilTransportResource;
use mithril_server_types::{StaticObject, DynamicObject, Deleted, VisibleObjects, Viewport};

#[derive(Default)]
pub struct RegionUpdateSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, RegionUpdateSystem> for RegionUpdateSystemDesc {
    fn build(self, world: &mut World) -> RegionUpdateSystem {
        <RegionUpdateSystem as System<'_>>::SystemData::setup(world); 
        RegionUpdateSystem
    }
}



pub struct RegionUpdateSystem;

impl<'a> System<'a> for RegionUpdateSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Viewport>,
        ReadStorage<'a, StaticObject>,
        ReadStorage<'a, DynamicObject>,
        ReadStorage<'a, Deleted>,
        WriteStorage<'a, VisibleObjects>,
        Write<'a, MithrilTransportResource>,
    );

    fn run(
        &mut self,
        (entities, position, _viewport, static_flag, dynamic_flag, deleted, mut visible, mut net): Self::SystemData,
    ) {

        // static objects
        let _ = (&entities, &position,  &mut visible)
            .par_join()
            .map(|(player, player_position, visible)| {
                let viewport = Viewport::new(*player_position);
                let visible_static: Vec<(Entity, &Position, bool)> = (&entities, &static_flag, &deleted, &position, (&visible.0).maybe())
                    .join()
                    .filter(|(_, _, _, pos, _)| viewport.contains(&pos))
                    .map(|(entity, _, _, pos, known)| (entity, pos, known.is_some()))
                    .collect();

                let visible_dynamic: Vec<(Entity, bool, &Position, bool)> = (&entities, &dynamic_flag, (&deleted).maybe(), &position, (&visible.0).maybe())
                    .join()
                    .filter(|(_, _, _, pos, _)| viewport.contains(&pos))
                    .map(|(entity, _, deleted, pos, known)| (entity, deleted.is_some(), pos, known.is_some()))
                    .collect();
            });

        // Send updates
        // (objects & !new & removed.maybe())
        //   .join()
        //     // Map to update/remove
        //       .collect()
        //
        //       (objects && new && !removed)
        //         .join()
        //           // Map to add
        //             .collect()
        //
        //             // Combine updates, limit to player view and send packet
        //             // If player is in a new region, skip removed and send clear packet before
        //             sending all adds
        //
        //             objects = (objects & !removed).join().collect(); // mass delete
    }
}
