use specs::{Component, VecStorage, Index};
use indexmap::set::IndexSet;
use specs::{Component, VecStorage, NullStorage};
use specs::world::Index;
use std::fmt::{self, Display};

pub struct Name(pub String);

impl Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Component for Name {
    type Storage = VecStorage<Self>;
}

pub struct VisiblePlayers(pub IndexSet<Index>);

impl Component for VisiblePlayers {
    type Storage = VecStorage<Self>;
}

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct NewPlayer;