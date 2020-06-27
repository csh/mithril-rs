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
        (entities, position, _viewport, mut visible_regions, static_flag, deleted, object_data, mut visible, mut net): Self::SystemData,
    ) {

        // static objects
        let player_with_updates: Vec<_> = (&entities, &position, &mut visible_regions, &mut visible)
            .par_join()
            .map(|(player, player_position, visible_regions, visible)| {
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

                let mut currently_visible: AHashSet<Region> = AHashSet::new();
                currently_visible.extend(deleted_static.keys().clone());
                currently_visible.extend(visible_dynamic.keys().clone());
                
                let updates: Vec<_> = currently_visible.iter()
                    .map(|region| {
                        let clear_region = if !visible_regions.0.contains(&region) {
                            Some(ClearRegion::new(*player_position, *region))
                        } else {
                            None
                        };

                        let static_updates = if let Some(deleted_static) = deleted_static.get(&region) {
                                EitherIter::Left(deleted_static.into_iter()
                                .filter(|(_, _, _, known)| !known)
                                .map(|(_entity, pos, data, _)| -> Box<dyn RegionUpdate> {                         
                                    match data {
                                        WorldObjectData::Object {id: _, object_type, orientation} => {
                                            Box::new(RemoveObject::new(*object_type, *orientation, pos).expect("Wrong orientation?"))
                                        },
                                        // This can't even be, but leaving it for completion
                                        WorldObjectData::TileItem(data) => {
                                            Box::new(RemoveTileItem::new(data.item, pos))
                                        }
                                    }
                                }))
                        } else {
                            EitherIter::Right(std::iter::empty::<Box<dyn RegionUpdate>>())
                        };

                        let dynamic_updates
                            = if let Some(visible_dynamic) = visible_dynamic.get(&region) {
                                EitherIter::Left(visible_dynamic.into_iter()
                                // Check if updated here
                                .filter(|(_, deleted, _, data, known)| Self::has_updates(*deleted, *known, data))
                                .map(|(_entity, deleted, pos, data, _)| {
                                    match data {
                                        WorldObjectData::Object {id, object_type, orientation } => {
                                            let b: Box<dyn RegionUpdate> = if *deleted {
                                                Box::new(RemoveObject::new(*object_type, *orientation, pos).expect("Bad orientation"))
                                            } else {
                                                Box::new(SendObject::new(*id, *object_type, *orientation, pos).expect("Bad orientation"))
                                            };
                                            b
                                        },
                                        WorldObjectData::TileItem(data) => {
                                            if *deleted {
                                                Box::new(RemoveTileItem::new(data.item, pos))
                                            } else {
                                                let b: Box<dyn RegionUpdate> = match data.get_old_amount() {
                                                    Some(old_amount) => Box::new(
                                                        UpdateTileItem::new(data.item, pos, data.get_amount(), old_amount)
                                                    ),
                                                    None => Box::new(
                                                        AddTileItem::new(data.item, data.get_amount(), pos)
                                                    ),
                                                };
                                                b
                                            }
                                        },    
                                    }
                                }))
                        } else {
                            EitherIter::Right(std::iter::empty::<Box<dyn RegionUpdate>>())
                        };
                        
                        
                        (clear_region, GroupedRegionUpdate::new(
                            *player_position,
                            *region,
                            static_updates.chain(dynamic_updates).collect(),
                        ))
                    }).collect();
                (player, currently_visible.clone(), updates)
            }).collect();

        let mut visible: AHashMap<_, _> = AHashMap::new();
        for (player, currently_visible, updates) in player_with_updates {
            for (clear_region, grouped_updates) in updates {
                if let Some(clear_region) = clear_region {
                    net.send(player, clear_region);    
                }
                net.send(player, grouped_updates);
            }
            visible.insert(player.id(), currently_visible);
        }

        (&entities, &mut visible_regions)
            .join()
            .for_each(|(player, mut visible_regions)| {
                if let Some(currently_visible) = visible.remove(&player.id()) {
                    visible_regions.0 = currently_visible;
                }
            });

        (&entities, &deleted, !&static_flag)
            .join()
            .for_each(|(entity, _, _)| {
                if let Err(_) = entities.delete(entity) {
                    log::info!("Failed to delete old entity?");
                }
            })
   }
}

impl RegionUpdateSystem {
    fn has_updates(known: bool, deleted: bool, data: &WorldObjectData) -> bool{
        if known != deleted {
            true
        } else {
            match data {
                WorldObjectData::Object{..} => false,
                WorldObjectData::TileItem(data) => {
                    match data.get_old_amount() {
                        Some(amount) => amount != data.get_amount(),
                        None => false,    
                    }
                }
            }
        }
    }
}

enum EitherIter<A, B, T> where A: Iterator<Item = T>, B: Iterator<Item = T> {
    Left(A),
    Right(B) 
}

impl<T, A, B> Iterator for EitherIter<A, B, T> where A: Iterator<Item = T>, B: Iterator<Item = T>{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(a) => a.next(),
            Self::Right(b) => b.next(),    
        }
    } 
}
