use amethyst::{
    core::SystemDesc,
    ecs::prelude::*
};
use ahash::{AHashMap, AHashSet};

use mithril_core::pos::{Position, Region};
use mithril_server_net::MithrilTransportResource;
use mithril_server_types::{StaticObject, Deleted, VisibleObjects, VisibleRegions, Viewport, WorldObjectData};
use mithril_core::net::packets::{
    RegionUpdate,
    AddTileItem,
    UpdateTileItem,
    SendObject,
    RemoveObject,
    RemoveTileItem,
    GroupedRegionUpdate,
    ClearRegion
};

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
        WriteStorage<'a, VisibleRegions>,
        ReadStorage<'a, StaticObject>,
        ReadStorage<'a, Deleted>,
        ReadStorage<'a, WorldObjectData>,
        WriteStorage<'a, VisibleObjects>,
        Write<'a, MithrilTransportResource>,
    );

    fn run(
        &mut self,
        (entities, position, _viewport, visible_regions, static_flag, deleted, object_data, mut visible, mut net): Self::SystemData,
    ) {

        // static objects
        let _ = (&entities, &position, &mut visible_regions, &mut visible)
            .par_join()
            .for_each(|(player, player_position, mut visible_regions, visible)| {
                let viewport = Viewport::new(*player_position);

                let deleted_static: AHashMap<Region, Vec<(Entity, &Position, &WorldObjectData, bool)>> = (&entities, &static_flag, &deleted, &position, &object_data, (&visible.0).maybe())
                    .join()
                    .filter(|(_, _, _, pos, _, _)| viewport.contains(&pos))
                    .map(|(entity, _, _, pos, data, known)| (entity, pos, data, known.is_some()))
                    .fold(AHashMap::new(), |mut map, data| {
                        let region: Region = data.1.into();
                        map.entry(region).or_insert_with(|| {
                            Vec::new()    
                        }).push(data);
                        map
                    });

                let visible_dynamic: AHashMap<Region, Vec<(Entity, bool, &Position, &WorldObjectData, bool)>> = (&entities, !&static_flag, (&deleted).maybe(), &position, &object_data, (&visible.0).maybe())
                    .join()
                    .filter(|(_, _, _, pos, _, _)| viewport.contains(&pos))
                    .map(|(entity, _, deleted, pos, data, known)| (entity, deleted.is_some(), pos, data, known.is_some()))
                    .fold(AHashMap::new(), |mut map, data| {
                        let region: Region = data.2.into();
                        map.entry(region).or_insert_with(|| {
                            Vec::new()    
                        }).push(data);
                        map
                    });

                let mut currently_visible = AHashSet::new();
                currently_visible.extend(deleted_static.keys().clone());
                currently_visible.extend(visible_dynamic.keys().clone());
                
                currently_visible.iter()
                    .for_each(|region| {
                        if !visible_regions.0.contains(region) {
                            net.send(player, ClearRegion::new(player_position, region));
                        }

                        let static_updates = if let Some(deleted_static) = deleted_static.get(region) {
                            deleted_static.iter()
                                .filter(|(_, _, _, known)| !known)
                                .map(|entity, pos, data, _| {                         
                                    match data {
                                        WorldObjectData::Object {object_type, orientation} => {
                                            RemoveObject::new(object_type, orientation, pos) 
                                        },
                                        // This can't even be, but leaving it for completion
                                        WorldObjectData::TileItem {item, amount} => {
                                            RemoveTileItem::new(item, pos)    
                                        }
                                    }
                                });
                        } else {
                            iter::empty::<Box<dyn RegionUpdate>>()
                        };

                        let dynamic_updates = if let Some(visible_dynamic) = visible_dynamic.get(region) {
                            visible_dynamic.iter()
                                // Check if updated here
                                .filter(|(_, deleted, _, _, known)| known != deleted)
                                .map(|(entity, deleted, pos, data, _)| {
                                    if deleted {
                                        match data {
                                            WorldObjectData::Object {object_type, orientation} => {
                                                RemoveObject::new(object_type, orientation, pos) 
                                            },
                                            WorldObjectData::TileItem {item, amount} => {
                                                RemoveTileItem::new(item, pos)    
                                            }
                                        }
                                    } else { // if new
                                        match data {
                                            WorldObjectData::Object {object_type, orientation} => {
                                                SendObject::new(0, object_type, orientation, pos)  
                                            },
                                            WorldObjectData::TileItem {item, amount} => {
                                                AddTileItem::new(item, amount, pos)
                                            }
                                        }
                                    }
                                })
                        } else {
                            iter::empty::<Box<dyn RegionUpdate>>()    
                        };
                        net.send(player, 
                            GroupedRegionUpdate::new(
                                player_position,
                                region,
                                static_updates.extend(dynamic_updates).collect()
                            )
                        );
                    });


                (*visible_regions).0 = currently_visible;
            });
   }
}
