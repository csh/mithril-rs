use indexmap::set::IndexSet;
use specs::world::Index;
use specs::{Component, NullStorage, VecStorage};

#[derive(Default, Debug)]
pub struct VisiblePlayers(pub IndexSet<Index>);

impl Component for VisiblePlayers {
    type Storage = VecStorage<Self>;
}

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct NewPlayer;
