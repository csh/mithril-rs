use mithril_core::pos::Position;
use specs::{Component, VecStorage};

#[derive(Debug, Default)]
pub struct PreviousPosition(pub Position);

impl PartialEq<Position> for PreviousPosition {
    fn eq(&self, other: &Position) -> bool {
        self.0.eq(other)
    }
}

impl Component for PreviousPosition {
    type Storage = VecStorage<Self>;
}