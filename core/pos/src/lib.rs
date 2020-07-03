use specs::{Component, VecStorage};
use std::fmt::Debug;
use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    None = -1,
    NorthWest = 0,
    North = 1,
    NorthEast = 2,
    West = 3,
    East = 4,
    SouthWest = 5,
    South = 6,
    SouthEast = 7,
}

impl Direction {
    pub fn to_orientation(&self) -> anyhow::Result<u8> {
        match &self {
            Self::West | Self::NorthWest => Ok(0),
            Self::North | Self::NorthEast => Ok(1),
            Self::East | Self::SouthEast => Ok(2),
            Self::South | Self::SouthWest => Ok(3),
            _ => Err(anyhow::anyhow!("This direction is not an orientation")),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct Region {
    pub x: i16,
    pub y: i16,
}

impl From<&Position> for Region {
    fn from(pos: &Position) -> Region {
        Region {
            x: pos.get_x() / 8,
            y: pos.get_y() / 8,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
    x: i16,
    y: i16,
    plane: u8,
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position::new_with_height(self.x + rhs.x, self.y + rhs.y, self.plane).expect("valid")
    }
}

impl Add<(i16, i16)> for Position {
    type Output = Position;

    fn add(self, rhs: (i16, i16)) -> Self::Output {
        Position::new_with_height(self.x + rhs.0, self.y + rhs.1, self.plane).expect("valid")
    }
}

impl Sub for Position {
    type Output = (i16, i16);

    fn sub(self, rhs: Self) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y)
    }
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Self::new_with_height(x, y, 0).expect("plane should never be out of bounds")
    }

    pub fn new_with_height(x: i16, y: i16, plane: u8) -> anyhow::Result<Self> {
        anyhow::ensure!(plane < 4, "plane out of bounds");
        Ok(Self { x, y, plane })
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

    pub fn direction_between(&self, other: Self) -> Direction {
        if *self == other {
            return Direction::None;
        }

        let delta_x = (other.x - self.x).signum();
        let delta_y = (other.y - self.y).signum();
        match delta_y {
            1 => match delta_x {
                1 => Direction::NorthEast,
                0 => Direction::North,
                -1 => Direction::NorthWest,
                _ => unreachable!(),
            },
            0 => match delta_x {
                1 => Direction::East,
                0 => Direction::None,
                -1 => Direction::West,
                _ => unreachable!(),
            },
            -1 => match delta_x {
                1 => Direction::SouthEast,
                0 => Direction::South,
                -1 => Direction::SouthWest,
                _ => unreachable!(),
            },
            _ => unreachable!(),
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
