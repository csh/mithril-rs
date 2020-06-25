use specs::{Component, NullStorage, VecStorage};

use mithril_core::net::packets::ObjectType;
use mithril_core::pos::Direction;

#[derive(Component)]
#[storage(VecStorage)]
pub enum WorldObjectData {
    Object {id: u16, object_type: ObjectType, orientation: Direction},
    TileItem {item: u16, amount: u16}    
}

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct StaticObject;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Deleted;
