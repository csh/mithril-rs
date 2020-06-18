use specs::{Component, VecStorage, NullStorage};
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

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct NewPlayer;