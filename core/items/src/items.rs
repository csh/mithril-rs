use ahash::AHashMap;

use mithril_fs::{defs::ItemDefinition, CacheFileSystem};

use crate::errors::ItemLookupError;
use crate::Item;

type Result<T> = std::result::Result<T, ItemLookupError>;

macro_rules! impl_delegate_getter {
    ($ty:ty, $getter:ident) => {
        pub fn $getter(&self, item: &Item) -> Result<$ty> {
            let def = self.try_get_definition(item.id())?;
            Ok(def.$getter())
        }
    };
}

pub struct Items {
    definitions: AHashMap<u16, ItemDefinition>,
}

impl Items {
    pub fn load(cache: &mut CacheFileSystem) -> Result<Self> {
        let mut definitions = AHashMap::new();
        // TODO: Discuss with team whether to change the return type of `load`
        let items = ItemDefinition::load(cache)?;
        for (id, item) in items.into_iter().enumerate() {
            definitions.insert(id as _, item);
        }

        Ok(Self { definitions })
    }

    pub fn create(&self, item_id: u16) -> Result<Item> {
        let def = self.try_get_definition(item_id)?;
        let item = if def.is_stackable() {
            Item::Stackable(item_id, 1)
        } else {
            Item::Single(item_id)
        };
        Ok(item)
    }

    pub fn create_with_quantity(&self, item_id: u16, quantity: u32) -> Result<Item> {
        let def = self.try_get_definition(item_id)?;
        if def.is_stackable() {
            Ok(Item::Stackable(item_id, quantity))
        } else {
            Err(ItemLookupError::NotStackable(item_id))
        }
    }

    impl_delegate_getter!(&String, name);
    impl_delegate_getter!(&String, examine_text);
    impl_delegate_getter!(bool, is_member_only);
    impl_delegate_getter!(bool, is_stackable);
    impl_delegate_getter!(bool, is_noted);
    impl_delegate_getter!(i32, value);

    pub fn ground_action(&self, item: &Item, index: usize) -> Result<Option<String>> {
        if index >= 5 {
            return Err(ItemLookupError::IndexOutOfBounds(index));
        }

        let def = self.try_get_definition(item.id())?;
        Ok(def.ground_action(index))
    }

    pub fn inventory_action(&self, item: &Item, index: usize) -> Result<Option<String>> {
        if index >= 5 {
            return Err(ItemLookupError::IndexOutOfBounds(index));
        }

        let def = self.try_get_definition(item.id())?;
        Ok(def.inventory_action(index))
    }

    fn try_get_definition(&self, item_id: u16) -> Result<&ItemDefinition> {
        self.definitions
            .get(&item_id)
            .ok_or_else(|| ItemLookupError::NotFound(item_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_items() {
        if ci_info::is_ci() {
            return;
        }

        let mut cache = CacheFileSystem::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../../cache"))
            .expect("cache");
        let items = Items::load(&mut cache).expect("items");

        mock_system(
            |(_, name, items)| {
                let adamant_2h = items.create(1317).expect("item exists");
                let item_name = match items.name(&adamant_2h) {
                    Ok(name) => name,
                    Err(e) => {
                        eprintln!("invalid item; {}", e);
                        return;
                    }
                };

                let action = match items.inventory_action(&adamant_2h, 1) {
                    Ok(Some(action)) => action,
                    Ok(None) => return,
                    Err(e) => {
                        eprintln!("bad item; {:?} {}", adamant_2h, e);
                        return;
                    }
                };

                if action == "Wield" {
                    println!("{} equips his {}", name, item_name);
                }
            },
            "Smrkn",
            &items,
        );
    }

    fn mock_system<F, S: Into<String>>(system: F, name: S, items: &Items)
    where
        F: FnOnce(((), String, &Items)) -> (),
    {
        system(((), name.into(), items))
    }
}
