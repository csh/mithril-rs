use ahash::AHashSet;
use hibitset::BitSet;
use indexmap::set::IndexSet;
use specs::world::Index;
use specs::{Component, NullStorage, VecStorage};

use mithril_core::pos::{Position, Region};

#[derive(Default, Debug)]
pub struct VisiblePlayers(pub IndexSet<Index>);

impl Component for VisiblePlayers {
    type Storage = VecStorage<Self>;
}

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct NewPlayer;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Default, Component, Debug)]
#[storage(VecStorage)]
pub struct VisibleObjects(pub BitSet);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct VisibleRegions(pub AHashSet<Region>);

const VIEWPORT_SIZE: i16 = 13 * 8;

#[derive(Default, Component)]
#[storage(VecStorage)]
pub struct Viewport {
    center: Position,
}

impl Viewport {
    pub fn new(position: Position) -> Self {
        Viewport { center: position }
    }

    pub fn contains(&self, position: &Position) -> bool {
        if position.get_plane() != self.center.get_plane() {
            return false;
        }

        let min_vx = (self.center.get_x() / 8 - 6) * 8;
        let min_vy = (self.center.get_y() / 8 - 6) * 8;
        let max_vx = min_vx + VIEWPORT_SIZE;
        let max_vy = min_vy + VIEWPORT_SIZE;

        (min_vx..=max_vx).contains(&position.get_x())
            && (min_vy..=max_vy).contains(&position.get_y())
    }
}
