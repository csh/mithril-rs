use indexmap::set::IndexSet;
use specs::{Component, VecStorage, NullStorage};
use specs::world::Index;

#[derive(Default, Debug)]
pub struct VisiblePlayers(pub IndexSet<Index>);

impl Component for VisiblePlayers {
    type Storage = VecStorage<Self>;
}

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct NewPlayer;