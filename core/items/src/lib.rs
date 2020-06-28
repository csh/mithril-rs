mod errors;
mod items;

pub use errors::ItemLookupError;
pub use items::Items;
use std::hash::Hash;

pub const MAX_ITEM_SIZE: u32 = i32::MAX as u32;

#[derive(Debug, Copy, Clone)]
pub enum Item {
    Single(u16),
    Stackable(u16, u32),
}

impl Item {
    pub fn id(self) -> u16 {
        match self {
            Self::Single(id) => id,
            Self::Stackable(id, _) => id,
        }
    }

    pub fn get_quantity(self) -> u32 {
        match self {
            Self::Single(_) => 1,
            Self::Stackable(_, quantity) => quantity,
        }
    }

    pub fn set_quantity(&mut self, new_quantity: u32) -> Result<(), ItemLookupError> {
        match self {
            Self::Single(id) => Err(ItemLookupError::NotStackable(*id)),
            Self::Stackable(_, ref mut quantity) => {
                *quantity = std::cmp::min(new_quantity, MAX_ITEM_SIZE);
                Ok(())
            }
        }
    }

    pub fn is_stackable(self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::Stackable(_, _) => true,
        }
    }
}

impl PartialEq<u16> for Item {
    fn eq(&self, other: &u16) -> bool {
        self.id() == *other
    }
}

impl Hash for Item {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u16(self.id());
        state.write_u32(self.get_quantity());
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.get_quantity() == other.get_quantity()
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.id() == other.id() {
            self.get_quantity().partial_cmp(&other.get_quantity())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparisons() {
        let coins_l = Item::Single(995);
        let coins_r = Item::Stackable(995, 20);
        assert!(coins_l < coins_r, "r contains more coins");
        assert!(coins_l != coins_r, "different quantity");
        assert!(coins_l == Item::Stackable(995, 1), "identical");
        assert!(coins_l != Item::Single(10), "different id");
        assert!(coins_l == 995, "same ID");
    }
}
