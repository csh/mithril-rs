use specs::{Component, NullStorage, VecStorage};

use mithril_core::net::packets::ObjectType;
use mithril_core::pos::Direction;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub enum WorldObjectData {
    Object {id: u16, object_type: ObjectType, orientation: Direction},
    TileItem(TileItemData)    
}

#[derive(Debug)]
pub struct TileItemData {
    pub item: u16,
    amount: u16,
    old_amount: Option<u16>    
}

impl TileItemData {
    pub fn new(item: u16, amount: u16) -> Self {
        TileItemData {
            item,
            amount,
            old_amount: None,    
        }
    }

    pub fn take(&mut self, amount: u16) -> bool {
        if amount == 0 || amount > self.amount {
            false
        } else {
            if self.old_amount.is_none() {
                self.old_amount = Some(self.amount);
            }
            self.amount -= amount;
            true
        }
    }

    pub fn get_amount(&self) -> u16 {
        self.amount
    }

    pub fn get_old_amount(&self) -> Option<u16> {
        self.old_amount
    }
}

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct StaticObject;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Deleted;
