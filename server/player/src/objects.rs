use ahash::{AHashMap, AHashSet};
use amethyst::{core::SystemDesc, ecs::prelude::*};

use mithril_core::net::packets::{
    AddTileItem, ClearRegion, GroupedRegionUpdate, RegionUpdate, RemoveObject, RemoveTileItem,
    SendObject, UpdateTileItem,
};
use mithril_core::pos::{Position, Region};
use mithril_server_net::MithrilTransportResource;
use mithril_server_types::{
    Deleted, StaticObject, Viewport, VisibleObjects, VisibleRegions, WorldObjectData,
};

#[derive(Default)]
pub struct RegionUpdateSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, RegionUpdateSystem> for RegionUpdateSystemDesc {
    fn build(self, world: &mut World) -> RegionUpdateSystem {
        <RegionUpdateSystem as System<'_>>::SystemData::setup(world);
        RegionUpdateSystem
    }
}

#[derive(SystemData)]
pub struct ObjectStorage<'a> {
    static_flag: ReadStorage<'a, StaticObject>,
    deleted: ReadStorage<'a, Deleted>,
    object_data: ReadStorage<'a, WorldObjectData>,
}

#[derive(SystemData)]
pub struct PlayerStorage<'a> {
    visible_regions: WriteStorage<'a, VisibleRegions>,
    visible_objects: WriteStorage<'a, VisibleObjects>,
}

pub struct RegionUpdateSystem;

impl<'a> System<'a> for RegionUpdateSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        PlayerStorage<'a>,
        ObjectStorage<'a>,
        Write<'a, MithrilTransportResource>,
    );

    fn run(&mut self, (entities, position, mut player, object, mut net): Self::SystemData) {
        let static_flag = object.static_flag;
        let deleted = object.deleted;
        let object_data = object.object_data;
        // static objects
        let player_with_updates: Vec<_> = (
            &entities,
            &position,
            &mut player.visible_regions,
            &mut player.visible_objects,
        )
            .par_join()
            .map(|(player, player_position, visible_regions, visible_objects)| {
                let viewport = Viewport::new(*player_position);
                let deleted_static: AHashMap<_, _> = (
                    &entities,
                    &static_flag,
                    (&deleted).maybe(),
                    &position,
                    &object_data,
                    (&visible_objects.0).maybe(),
                )
                    .join()
                    .filter(|(_, _, _, pos, _, _)| viewport.contains(&pos))
                    .map(|(entity, _, deleted, pos, data, known)| (entity, deleted.is_some(), pos, data, known.is_some()))
                    .fold(AHashMap::new(), |mut map, data| {
                        let region: Region = data.2.into();
                        map.entry(region).or_insert_with(Vec::new).push(data);
                        map
                    });

                let visible_dynamic: AHashMap<_, _> = (
                    &entities,
                    !&static_flag,
                    (&deleted).maybe(),
                    &position,
                    &object_data,
                    (&visible_objects.0).maybe(),
                )
                    .join()
                    .filter(|(_, _, _, pos, _, _)| viewport.contains(&pos))
                    .map(|(entity, _, deleted, pos, data, known)| {
                        (entity, deleted.is_some(), pos, data, known.is_some())
                    })
                    .fold(AHashMap::new(), |mut map, data| {
                        let region: Region = data.2.into();
                        map.entry(region).or_insert_with(Vec::new).push(data);
                        map
                    });

                let mut currently_visible_regions: AHashSet<Region> = AHashSet::new();
                currently_visible_regions.extend(deleted_static.keys().clone());
                currently_visible_regions.extend(visible_dynamic.keys().clone());

                let updates: Vec<_> = currently_visible_regions
                    .iter()
                    .map(|region| {
                        let clear_region = if !visible_regions.0.contains(&region) {
                            Some(ClearRegion::new(*player_position, *region))
                        } else {
                            None
                        };

                        let static_updates = if let Some(deleted_static) =
                            deleted_static.get(&region)
                        {
                            EitherIter::Left(
                                deleted_static.iter().filter(|(_, deleted, _, _, known)| *deleted && *known).map(
                                    |(_entity, _, pos, data, _)| -> RegionUpdate {
                                        match data {
                                            WorldObjectData::Object {
                                                id: _,
                                                object_type,
                                                orientation,
                                            } => RemoveObject::new(*object_type, *orientation, pos)
                                                    .expect("Wrong orientation?")
                                                    .into(),
                                            // This can't even be, but leaving it for completion
                                            WorldObjectData::TileItem(data) => {
                                                RemoveTileItem::new(data.item, pos).into()
                                            }
                                        }
                                    },
                                ),
                            )
                        } else {
                            EitherIter::Right(std::iter::empty::<RegionUpdate>())
                        };

                        let dynamic_updates =
                            if let Some(visible_dynamic) = visible_dynamic.get(&region) {
                                EitherIter::Left(
                                    visible_dynamic
                                        .iter()
                                        // Check if updated here
                                        .filter(|(_, deleted, _, data, known)| {
                                            Self::has_updates(*deleted, *known, data)
                                        })
                                        .map(|(_entity, deleted, pos, data, _)| match data {
                                            WorldObjectData::Object {
                                                id,
                                                object_type,
                                                orientation,
                                            } => {
                                                if *deleted {
                                                    RemoveObject::new(
                                                        *object_type,
                                                        *orientation,
                                                        pos,
                                                    )
                                                    .expect("Bad orientation")
                                                    .into()
                                                } else {
                                                    SendObject::new(
                                                        *id,
                                                        *object_type,
                                                        *orientation,
                                                        pos,
                                                    )
                                                    .expect("Bad orientation")
                                                    .into()
                                                }
                                            }
                                            WorldObjectData::TileItem(data) => {
                                                if *deleted {
                                                    RemoveTileItem::new(data.item, pos).into()
                                                } else { 
                                                    match data.get_old_amount() {
                                                        Some(old_amount) => {
                                                            UpdateTileItem::new(
                                                                data.item,
                                                                pos,
                                                                data.get_amount(),
                                                                old_amount,
                                                            ).into()
                                                        },
                                                        None => AddTileItem::new(
                                                                data.item,
                                                                data.get_amount(),
                                                                pos,
                                                            ).into()
                                                    }
                                                }
                                            }
                                        }),
                                )
                            } else {
                                EitherIter::Right(std::iter::empty::<RegionUpdate>())
                            };

                        let updates: Vec<_> = static_updates.chain(dynamic_updates).collect();

                        let grouped_update = if !updates.is_empty() {
                            Some(GroupedRegionUpdate::new(
                                *player_position,
                                *region,
                            ).add_all(updates))
                        } else {
                            None    
                        };

                        (
                            clear_region,
                            grouped_update,
                        )
                    })
                    .collect();

                let ds = deleted_static.values().flat_map(|val| val.iter())
                            .filter(|(_, deleted, ..)| !deleted)
                            .map(|(entity, ..)| entity.id());

                let vd = visible_dynamic.values().flat_map(|val| val.iter())
                            .filter(|(_, deleted, ..)| !deleted)
                            .map(|(entity, ..)| entity.id());

                let new_visible_objects = ds.chain(vd).fold(BitSet::new(), |mut bitset, id| {
                    bitset.add(id);
                    bitset    
                });

                (player, currently_visible_regions, new_visible_objects, updates)
            })
            .collect();

        let mut visible: AHashMap<_, _> = AHashMap::new();
        for (player, visible_regions, visible_objects, updates) in player_with_updates {
            for (clear_region, grouped_updates) in updates {
                if let Some(clear_region) = clear_region {
                    net.send(player, clear_region);
                }
                if let Some(grouped_updates) = grouped_updates {
                    net.send(player, grouped_updates);
                }
            }
            visible.insert(player.id(), (visible_regions, visible_objects));
        }

        (&entities, &mut player.visible_regions, &mut player.visible_objects).join().for_each(
            |(player, mut visible_regions, mut visible_objects)| {
                if let Some((l_visible_regions, l_visible_objects)) = visible.remove(&player.id()) {
                    visible_regions.0 = l_visible_regions;
                    visible_objects.0 = l_visible_objects;
                }
            },
        );

        (&entities, &deleted, !&static_flag)
            .join()
            .for_each(|(entity, _, _)| {
                if entities.delete(entity).is_err() {
                    log::info!("Failed to delete old entity?");
                }
            })
    }
}

impl RegionUpdateSystem {
    fn has_updates(known: bool, deleted: bool, data: &WorldObjectData) -> bool {
        if known == deleted {
            true
        } else {
            match data {
                WorldObjectData::Object { .. } => false,
                WorldObjectData::TileItem(data) => match data.get_old_amount() {
                    Some(amount) => amount != data.get_amount(),
                    None => false,
                },
            }
        }
    }
}

enum EitherIter<A, B, T>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    Left(A),
    Right(B),
}

impl<T, A, B> Iterator for EitherIter<A, B, T>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(a) => a.next(),
            Self::Right(b) => b.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use amethyst_test::prelude::*;
    use amethyst::{GameData, StateEvent, StateEventReader};
    use mithril_core::{
        net::packets::{
            ObjectType, PacketEvent, GameplayEvent
        },
        pos::*
    };
    use mithril_server_types::components::TileItemData;

    use super::*;
    
    fn bootstrap() -> AmethystApplication<GameData<'static, 'static>, StateEvent, StateEventReader> {
        let mut logger = amethyst::LoggerConfig::default();
        logger.level_filter = log::LevelFilter::Debug;
        amethyst::start_logger(logger);
        AmethystApplication::blank()
            .with_setup(|world| {
                world.insert(MithrilTransportResource::default()); 
            })
            .with_system_desc(RegionUpdateSystemDesc, "region_update", &[])
            .with_effect(|world| { 
                let static_object = world.create_entity()
                    .with(Position::default() + (1, 1))
                    .with(StaticObject)
                    .with(WorldObjectData::Object{id: 0, object_type: ObjectType::Interactable, orientation: Direction::North})
                    .build();

                let dynamic_object = world.create_entity()
                    .with(Position::default() + (2, 2))
                    .with(WorldObjectData::Object{id: 0, object_type: ObjectType::Interactable, orientation: Direction::North})
                    .build();

                let tile_item = world.create_entity()
                    .with(Position::default() + (3, 3))
                    .with(WorldObjectData::TileItem(TileItemData::new(20, 1)))
                    .build();

                let mut bitset = BitSet::new();
                // known static objects are deleted
                bitset.add(static_object.id());
                bitset.add(dynamic_object.id());
                bitset.add(tile_item.id());

                let mut visible_regions = AHashSet::new();
                visible_regions.insert((&Position::default()).into());
                visible_regions.insert((&(Position::default() + (3,3))).into());

                world.create_entity()
                    .with(Position::default())
                    .with(VisibleRegions(visible_regions))
                    .with(VisibleObjects(bitset))
                    .build(); 
            })
            
    }

    #[test]
    fn test_noop() {
        bootstrap()
            .with_assertion(|world| {
                let net = world.read_resource::<MithrilTransportResource>();
                let player: Entity = world.entities().entity(3);
                let component = world.read_component::<VisibleRegions>();
                let visible_regions: &VisibleRegions = component.get(player).unwrap();
                let mut should_be_regions = AHashSet::new();
                should_be_regions.insert((&Position::default()).into());
                should_be_regions.insert((&(Position::default() + (3, 3))).into());

                assert!(net.queued_packets().is_empty(), "There should be no packets");
                assert_eq!(visible_regions.0, should_be_regions);
            })
            .run().expect("Running system failed"); 
    }

    #[test]
    fn test_static_object_delete() {
        bootstrap()
            .with_effect(|world| {
                let static_object = (&world.entities(), &world.read_storage::<StaticObject>())
                    .join()
                    .map(|(entity, _)| entity)
                    .nth(0).unwrap();
                world.write_component::<Deleted>().insert(static_object, Deleted).expect("Failed to add deleted flag"); 
            })
            .with_assertion(|world| {
                // Never delete static objects
                let static_object_not_deleted = (&world.entities(), &world.read_storage::<StaticObject>())
                    .join()
                    .nth(0)
                    .is_some();
                assert!(static_object_not_deleted);
                
                let net = world.read_resource::<MithrilTransportResource>();
                let static_object_pos = Position::default() + (1, 1);
                assert_eq!(net.queued_packets().len(), 1);
                let (entity, packet_event) = net.queued_packets().front().unwrap(); 
                assert_eq!(entity.id(), 3); // Is our player
                assert!(packet_event.is_gameplay()); // Gameplay packet
                if let PacketEvent::Gameplay(GameplayEvent::GroupedRegionUpdate(packet)) = packet_event {
                    assert_eq!(packet, &GroupedRegionUpdate {
                        region: (&static_object_pos).into(),
                        viewport_center: Position::default(),
                        updates: vec![
                            RemoveObject::new(ObjectType::Interactable, Direction::North, &static_object_pos).expect("Invalid direction").into()
                        ],
                    });
                } 
            })
            .run().expect("Running system failed");
    }

    #[test]
    fn test_dynamic_object_delete() {
        bootstrap()
            .with_effect(|world| {
                let dynamic_object = (&world.entities(), 
                    !&world.read_storage::<StaticObject>(),
                    &world.read_storage::<WorldObjectData>()
                )
                    .join()
                    .filter(|(_, _, data)| {
                        if let WorldObjectData::Object {..} = data {
                            true    
                        } else {
                            false    
                        }
                    })
                    .map(|(entity, _, _)| entity)
                    .nth(0).unwrap();
                world.write_component::<Deleted>().insert(dynamic_object, Deleted).expect("Failed to add deleted flag"); 
            })
            .with_assertion(|world| {
                // Never delete static objects
                world.maintain();
                let dynamic_object_deleted = (&world.entities(),
                    !&world.read_storage::<StaticObject>(),
                    &world.read_storage::<WorldObjectData>()
                )
                    .join()
                    .filter(|(_, _, data)| {
                        if let WorldObjectData::Object {..} = data {
                            true    
                        } else {
                            false
                        }
                    })
                    .nth(0)
                    .is_none();
                assert!(dynamic_object_deleted);
                
                let net = world.read_resource::<MithrilTransportResource>();
                let dynamic_object_pos = Position::default() + (2, 2);
                assert_eq!(net.queued_packets().len(), 1);
                let (entity, packet_event) = net.queued_packets().front().unwrap(); 
                assert_eq!(entity.id(), 3); // Is our player
                assert!(packet_event.is_gameplay()); // Gameplay packet
                if let PacketEvent::Gameplay(GameplayEvent::GroupedRegionUpdate(packet)) = packet_event {
                    assert_eq!(packet, &GroupedRegionUpdate {
                        region: (&dynamic_object_pos).into(),
                        viewport_center: Position::default(),
                        updates: vec![
                            RemoveObject::new(ObjectType::Interactable, Direction::North, &dynamic_object_pos).expect("Invalid direction").into()
                        ],
                    });
                } 
            })
            .run().expect("Running system failed");
    }

    #[test]
    fn test_tile_item_remove() {
        bootstrap()
            .with_effect(|world| {
                let dynamic_object = (&world.entities(), 
                    !&world.read_storage::<StaticObject>(),
                    &world.read_storage::<WorldObjectData>()
                )
                    .join()
                    .filter(|(_, _, data)| {
                        if let WorldObjectData::TileItem(_)= data {
                            true    
                        } else {
                            false    
                        }
                    })
                    .map(|(entity, _, _)| entity)
                    .nth(0).unwrap();
                world.write_component::<Deleted>().insert(dynamic_object, Deleted).expect("Failed to add deleted flag"); 
            })
            .with_assertion(|world| {
                // Never delete static objects
                world.maintain();
                let dynamic_object_deleted = (&world.entities(),
                    !&world.read_storage::<StaticObject>(),
                    &world.read_storage::<WorldObjectData>()
                )
                    .join()
                    .filter(|(_, _, data)| {
                        if let WorldObjectData::TileItem(_) = data {
                            true
                        } else {
                            false
                        }
                    })
                    .nth(0)
                    .is_none();
                assert!(dynamic_object_deleted);
                
                let net = world.read_resource::<MithrilTransportResource>();
                let tile_item_pos = Position::default() + (3, 3);
                assert_eq!(net.queued_packets().len(), 1);
                let (entity, packet_event) = net.queued_packets().front().unwrap(); 
                assert_eq!(entity.id(), 3); // Is our player
                assert!(packet_event.is_gameplay()); // Gameplay packet
                if let PacketEvent::Gameplay(GameplayEvent::GroupedRegionUpdate(packet)) = packet_event {
                    assert_eq!(packet, &GroupedRegionUpdate {
                        region: (&tile_item_pos).into(),
                        viewport_center: Position::default(),
                        updates: vec![
                            RemoveTileItem::new(20, &tile_item_pos).into()
                        ],
                    });
                } 
            })
            .run().expect("Running system failed");
    }

    #[test]
    fn test_dynamic_object_add() {
        bootstrap()
            .with_effect(|world| {
                world.create_entity()
                    .with(Position::default() + (4,4))
                    .with(WorldObjectData::Object {id: 1, object_type: ObjectType::DiagonalWall, orientation: Direction::West})
                    .build();
            })
            .with_assertion(|world| {
                // Never delete static objects
                world.maintain();
                let entities = world.entities();
                let position_component = world.read_storage::<Position>();
                let data_component = world.read_storage::<WorldObjectData>();
                let (new_object_id, position, object_type, orientation) = (
                    &entities,
                    &position_component,
                    &data_component
                )
                    .join()
                    .filter_map(|(entity, position, data)| {
                        if let WorldObjectData::Object {id: 1, object_type, orientation} = data {
                            Some((entity.id(), position, object_type, orientation))
                        } else {
                            None
                        }
                    })
                    .nth(0).unwrap();
                let visible_objects_component = world.read_storage::<VisibleObjects>();
                let visible_objects = (&world.entities(),
                    &visible_objects_component,
                )
                    .join()
                    .map(|(_, vis)| vis)
                    .nth(0)
                    .unwrap();
                assert!(visible_objects.0.contains(new_object_id));
                
                let net = world.read_resource::<MithrilTransportResource>();
                assert_eq!(net.queued_packets().len(), 1);
                let (entity, packet_event) = net.queued_packets().front().unwrap(); 
                assert_eq!(entity.id(), 3); // Is our player
                assert!(packet_event.is_gameplay()); // Gameplay packet
                if let PacketEvent::Gameplay(GameplayEvent::GroupedRegionUpdate(packet)) = packet_event {
                    assert_eq!(packet, &GroupedRegionUpdate {
                        region: (position).into(),
                        viewport_center: Position::default(),
                        updates: vec![
                            SendObject::new(1, *object_type, *orientation, &position)
                                .expect("Invalid orientation")
                                .into()
                        ],
                    });
                } 
            })
            .run().expect("Running system failed");

    }

    #[test]
    fn test_tile_item_add() {
        bootstrap()
            .with_effect(|world| {
                world.create_entity()
                    .with(Position::default() + (4,4))
                    .with(WorldObjectData::TileItem(TileItemData::new(21, 5)))
                    .build();
            })
            .with_assertion(|world| {
                // Never delete static objects
                world.maintain();
                let entities = world.entities();
                let position_component = world.read_storage::<Position>();
                let data_component = world.read_storage::<WorldObjectData>();
                let (new_object_id, position, item_data) = (
                    &entities,
                    &position_component,
                    &data_component
                )
                    .join()
                    .filter_map(|(entity, position, data)| {
                        if let WorldObjectData::TileItem(tile_item_data) = data {
                            if tile_item_data.item == 21 {
                                Some((entity.id(), position, tile_item_data))
                            } else {
                                None    
                            }
                        } else {
                            None
                        }
                    })
                    .nth(0).unwrap();
                let visible_objects_component = world.read_storage::<VisibleObjects>();
                let visible_objects = (&world.entities(),
                    &visible_objects_component,
                )
                    .join()
                    .map(|(_, vis)| vis)
                    .nth(0)
                    .unwrap();
                assert!(visible_objects.0.contains(new_object_id));
                
                let net = world.read_resource::<MithrilTransportResource>();
                assert_eq!(net.queued_packets().len(), 1);
                let (entity, packet_event) = net.queued_packets().front().unwrap(); 
                assert_eq!(entity.id(), 3); // Is our player
                assert!(packet_event.is_gameplay()); // Gameplay packet
                if let PacketEvent::Gameplay(GameplayEvent::GroupedRegionUpdate(packet)) = packet_event {
                    assert_eq!(packet, &GroupedRegionUpdate {
                        region: (position).into(),
                        viewport_center: Position::default(),
                        updates: vec![
                            AddTileItem::new(item_data.item, item_data.get_amount(), position).into()
                        ],
                    });
                } 
            })
            .run().expect("Running system failed");
    }

    #[test]
    fn test_tile_item_update() {
        bootstrap()
            .with_effect(|world| {
                let mut item_data = TileItemData::new(21, 5);
                // Make it 3
                item_data.take(2);
                // create the entity
                let item = world.create_entity()
                    .with(Position::default() + (4,4))
                    .with(WorldObjectData::TileItem(item_data))
                    .build();
                // In normal cases, we know the entity
                let entities = world.entities();
                let mut visible_objects_component = world.write_storage::<VisibleObjects>();
                let visible_objects = visible_objects_component.get_mut(entities.entity(3)).unwrap();
                visible_objects.0.add(item.id());
            })
            .with_assertion(|world| {
                // Never delete static objects
                world.maintain();
                let entities = world.entities();
                let position_component = world.read_storage::<Position>();
                let data_component = world.read_storage::<WorldObjectData>();
                let (new_object_id, position, item_data) = (
                    &entities,
                    &position_component,
                    &data_component
                )
                    .join()
                    .filter_map(|(entity, position, data)| {
                        if let WorldObjectData::TileItem(tile_item_data) = data {
                            if tile_item_data.item == 21 {
                                Some((entity.id(), position, tile_item_data))
                            } else {
                                None    
                            }
                        } else {
                            None
                        }
                    })
                    .nth(0).unwrap();
                let visible_objects_component = world.read_storage::<VisibleObjects>();
                let visible_objects = (&world.entities(),
                    &visible_objects_component,
                )
                    .join()
                    .map(|(_, vis)| vis)
                    .nth(0)
                    .unwrap();
                assert!(visible_objects.0.contains(new_object_id));
                
                let net = world.read_resource::<MithrilTransportResource>();
                assert_eq!(net.queued_packets().len(), 1);
                let (entity, packet_event) = net.queued_packets().front().unwrap(); 
                assert_eq!(entity.id(), 3); // Is our player
                assert!(packet_event.is_gameplay()); // Gameplay packet
                if let PacketEvent::Gameplay(GameplayEvent::GroupedRegionUpdate(packet)) = packet_event {
                    assert_eq!(packet, &GroupedRegionUpdate {
                        region: (position).into(),
                        viewport_center: Position::default(),
                        updates: vec![
                            UpdateTileItem::new(item_data.item, position, item_data.get_amount(), item_data.get_old_amount().unwrap()).into()
                        ],
                    });
                } 
            })
            .run().expect("Running system failed");
    }

    #[test]
    fn test_player_movement_newregion() {
         bootstrap()
            .with_effect(|world| {
                world.create_entity()
                    .with(Position::default())
                    .with(VisibleRegions(AHashSet::new()))
                    .with(VisibleObjects(BitSet::new()))
                    .build();    
            })
            .with_assertion(|world| {
                let net = world.read_resource::<MithrilTransportResource>();
                let player = world.entities().entity(3);
                let other = world.entities().entity(4);
                let regions_component = world.read_component::<VisibleRegions>();
                let visible_regions = regions_component.get(player).unwrap();
                let objects_component = world.read_component::<VisibleObjects>();
                // This should be the same as our new player
                let visible_objects = objects_component.get(player).unwrap();
                let mut should_be_regions = AHashSet::new();
                let region_a = (&Position::default()).into();
                let region_b = (&(Position::default() + (3, 3))).into();
                should_be_regions.insert(region_a);
                should_be_regions.insert(region_b);
 
                assert_eq!(net.queued_packets().len(), 4, "There should be 4 packets");                 
                let packets_by_player: AHashMap<_, _> = net.queued_packets().iter()
                    .filter_map(|event| {
                        if let PacketEvent::Gameplay(packet_event) = &event.1 {
                            Some((event.0.id(), packet_event))
                        } else {
                            None    
                        }
                    })
                    .fold(AHashMap::new(), |mut map, (id, packet_event)| {
                        map.entry(id).or_insert_with(Vec::new).push(packet_event);
                        map
                    });

                // Existing player
                assert_eq!(visible_regions.0, should_be_regions);
                assert!(packets_by_player.get(&player.id()).is_none());

                // New player
                let packets = packets_by_player.get(&other.id()).unwrap();
                assert_eq!(packets.len(), 4);
                #[cfg(feature = "test-equality")]
                {
                    let expected_packets: Vec<GameplayEvent> = vec![
                        ClearRegion::new(Position::default(), region_a).into(),
                        GroupedRegionUpdate::new(Position::default(), region_a)
                            .add_update(
                                SendObject::new(0, ObjectType::Interactable, Direction::North, &(Position::default() + (2,2)))
                                    .expect("Invalid direction"))
                            .into(),
                        ClearRegion::new(Position::default(), region_b).into(),
                        GroupedRegionUpdate::new(Position::default(), region_b)
                            .add_update(
                                AddTileItem::new(20, 1, &(Position::default() + (3, 3)))
                            ).into()
                    ];

                    let mut ep: Vec<&GameplayEvent> = expected_packets.iter().collect();
                    let first = packets == &ep;
                    dbg!(&packets);
                    dbg!(&ep);
                    let (c_a, u_a, c_b, u_b) = (ep[0], ep[1], ep[2], ep[3]);
                    ep[0] = c_b;
                    ep[1] = u_b;
                    ep[2] = c_a;
                    ep[3] = u_a;
                    dbg!(&ep);
                    let second = packets == &ep;
                    assert!(first || second, "Packet ordering is wrong");
                }
                assert_eq!(regions_component.get(other).unwrap().0, visible_regions.0);
                assert_eq!(objects_component.get(other).unwrap().0, visible_objects.0);

                
            })
            .run().expect("Running system failed"); 
    }

    #[test]
    fn test_player_movement_oldregion() {
        bootstrap()
            .with_effect(|world| {
                let mut position_component = world.write_component::<Position>();
                let player = world.entities().entity(3);
                let current = *position_component.get(player).unwrap();
                position_component.insert(player, current + (105, 105))
                    .expect("Failed to update position");
            })
            .with_assertion(|world| {
                let net = world.read_resource::<MithrilTransportResource>();
                let player = world.entities().entity(3);
                let component = world.read_component::<VisibleRegions>();
                let visible_regions = component.get(player).unwrap();
                let component = world.read_component::<VisibleObjects>();
                let visible_objects = component.get(player).unwrap();
                let should_be_regions = AHashSet::new();
                let should_be_objects = BitSet::new();
                assert!(net.queued_packets().is_empty(), "There should be no packets");
                assert_eq!(visible_regions.0, should_be_regions);
                assert_eq!(visible_objects.0, should_be_objects);
            })
            .run().expect("Running system failed"); 
    }
}
