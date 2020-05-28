use std::fmt::Debug;
use specs::{Component, VecStorage};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Position {
    x: i16,
    y: i16,
    plane: u8
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Self::new_with_height(x, y, 0).expect("plane should never be out of bounds")
    }

    pub fn new_with_height(x: i16, y: i16, plane: u8) -> anyhow::Result<Self> {
        anyhow::ensure!(plane < 4, "plane out of bounds");
        Ok(Self {
            x,
            y,
            plane
        })
    }

    pub fn get_x(&self) -> i16 {
        self.x
    }

    pub fn get_y(&self) -> i16 {
        self.y
    }

    pub fn get_plane(&self) -> u8 {
        self.plane
    }

    pub fn get_region_x(&self) -> i16 {
        self.x / 8 - 6
    }

    pub fn get_region_y(&self) -> i16 {
        self.y / 8 - 6
    }

    pub fn get_relative(&self, other: Self) -> (u8, u8) {
        let local_x = self.get_x() - other.get_region_x() * 8;
        let local_y = self.get_y() - other.get_region_y() * 8;
        (local_x as u8, local_y as u8)
    }

    pub fn within_distance(&self, other: Self, distance: i16) -> bool {
        if other.plane != self.plane {
            false
        } else {
            let delta_x = (self.x - other.x).abs();
            let delta_y = (self.y - other.y).abs();
            delta_x <= distance && delta_y <= distance
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        // Tutorial island
        Position::new(3093, 3104)
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}