use specs::{Component, NullStorage, VecStorage};

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct StaticObject;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct DynamicObject;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Deleted;
