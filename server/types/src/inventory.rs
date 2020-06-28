use mithril_core::items::{Item, MAX_ITEM_SIZE};
use std::ops::Index;

#[derive(Debug)]
pub enum StackingOption {
    Always,
    OnlyStackable,
}

impl StackingOption {
    fn convert(&self, item: Item) -> Item {
        match (self, item) {
            (StackingOption::Always, Item::Single(id)) => Item::Stackable(id, 1),
            (StackingOption::Always, Item::Stackable(_, _))
            | (StackingOption::OnlyStackable, Item::Single(_))
            | (StackingOption::OnlyStackable, Item::Stackable(_, _)) => item,
        }
    }
}

#[derive(Debug)]
pub struct Inventory {
    items: Vec<Option<Item>>,
    items_updated: bool,
    stacking_option: StackingOption,
}

impl Inventory {
    pub fn with_capacity(capacity: usize, stacking_option: StackingOption) -> Self {
        Self {
            items: vec![None; capacity],
            items_updated: false,
            stacking_option,
        }
    }

    pub fn add(&mut self, item: Item) -> bool {
        let item = self.stacking_option.convert(item);
        if item.is_stackable() {
            self.add_stackable(item)
        } else {
            self.add_internal(item)
        }
    }

    pub fn set(&mut self, slot: usize, item: Item) -> Option<Item> {
        if self.items.capacity() < slot {
            return None;
        }

        let old = self.items[slot];
        match old {
            Some(old) if old != item => {
                self.items[slot] = Some(item);
                self.items_updated = true;
            }
            None => {
                self.items[slot] = Some(item);
                self.items_updated = true;
            }
            _ => {}
        }
        old
    }

    pub fn has(&self, item: Item) -> bool {
        self.items.iter().any(|other| match other {
            Some(other) => other >= &item,
            None => false,
        })
    }

    pub fn has_changes(&self) -> bool {
        self.items_updated
    }

    pub fn clear_changes(&mut self) {
        self.items_updated = false;
    }

    fn add_internal(&mut self, item: Item) -> bool {
        for slot_item in self.items.iter_mut() {
            if slot_item.is_none() {
                *slot_item = Some(item);
                self.items_updated = true;
                return true;
            }
        }
        false
    }

    fn add_stackable(&mut self, item: Item) -> bool {
        debug_assert!(
            item.is_stackable(),
            "method should not be called unless item is stackable"
        );
        for slot in self.items.iter_mut() {
            if let Some(other) = slot {
                if other.id() != item.id() {
                    continue;
                }
                let sum = other.get_quantity() + item.get_quantity();
                if sum > MAX_ITEM_SIZE {
                    return false;
                }
                other.set_quantity(sum).expect("item is stackable");
                self.items_updated = true;
                return true;
            }
        }
        self.add_internal(item)
    }
}

impl Index<usize> for Inventory {
    type Output = Option<Item>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bank() {
        let mut inventory = Inventory::with_capacity(1, StackingOption::Always);
        for _ in 0..28 {
            assert!(inventory.add(Item::Single(995)), "all items are stackable");
        }
        assert_eq!(
            inventory[0],
            Some(Item::Stackable(995, 28)),
            "inventory always stacks items"
        );
        assert!(inventory.has_changes(), "inventory has changed");
    }

    #[test]
    fn test_player_inventory() {
        let mut inventory = Inventory::with_capacity(28, StackingOption::OnlyStackable);
        for _ in 0..=28 {
            assert!(
                inventory.add(Item::Stackable(995, 1)),
                "item is stackable; should not fail"
            );
        }
        for _ in 0..27 {
            assert!(inventory.add(Item::Single(0)), "inventory is not full");
        }
        assert!(inventory.has_changes(), "inventory has changed");
        assert_eq!(
            inventory[0],
            Some(Item::Stackable(995, 29)),
            "item is stackable"
        );
    }

    #[test]
    fn test_has() {
        let mut inventory = Inventory::with_capacity(1, StackingOption::OnlyStackable);
        inventory.set(0, Item::Stackable(995, 5000));
        assert!(
            inventory.has(Item::Single(995)),
            "inventory contains at least 1 coin"
        );
    }
}
