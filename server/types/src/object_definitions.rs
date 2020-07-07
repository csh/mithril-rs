use ahash::AHashMap;

use mithril_core::fs::defs::ObjectDefinition;

pub struct ObjectDefinitions {
    definitions: AHashMap<u16, ObjectDefinition>,
}

impl ObjectDefinitions {
    pub fn new(definitions: AHashMap<u16, ObjectDefinition>) -> Self {
        ObjectDefinitions {
            definitions,    
        }
    }

    pub fn get<'a>(&'a self, id: u16) -> Option<&'a ObjectDefinition> {
        self.definitions.get(&id) 
    }
}
