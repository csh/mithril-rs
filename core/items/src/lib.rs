mod errors;
mod items;

pub use errors::ItemLookupError;
pub use items::Items;

#[derive(Debug, Hash)]
pub enum Item {
    Single(u16),
    Stackable(u16, u32),
}

impl Item {
    pub fn id(&self) -> u16 {
        match *self {
            Self::Single(id) => id,
            Self::Stackable(id, _) => id,
        }
    }

    pub fn get_quantity(&self) -> u32 {
        match self {
            Self::Single(_) => 1,
            Self::Stackable(_, quantity) => *quantity,
        }
    }

    pub fn set_quantity(&mut self, new_quantity: u32) -> Result<(), ItemLookupError> {
        match self {
            Self::Single(id) => Err(ItemLookupError::NotStackable(*id)),
            Self::Stackable(_, ref mut quantity) => {
                // Maximum is i32::MAX but we also don't require negative values.
                *quantity = std::cmp::min(new_quantity, i32::MAX as u32);
                Ok(())
            }
        }
    }
}

impl PartialEq<u16> for Item {
    fn eq(&self, other: &u16) -> bool {
        match self {
            Item::Single(id) => id == other,
            Item::Stackable(id, _) => id == other,
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        let (left_id, left_quantity) = match *self {
            Item::Single(id) => (id, 1),
            Item::Stackable(id, quantity) => (id, quantity),
        };

        let (right_id, right_quantity) = match *other {
            Item::Single(id) => (id, 1),
            Item::Stackable(id, quantity) => (id, quantity),
        };
        left_id == right_id && left_quantity == right_quantity
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let (left_id, left_quantity) = match *self {
            Item::Single(id) => (id, 1),
            Item::Stackable(id, quantity) => (id, quantity),
        };

        let (right_id, right_quantity) = match *other {
            Item::Single(id) => (id, 1),
            Item::Stackable(id, quantity) => (id, quantity),
        };

        if left_id == right_id {
            left_quantity.partial_cmp(&right_quantity)
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
