use mithril_core::pos::Position;
use specs::{Component, VecStorage};
use std::collections::VecDeque;

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

#[derive(Debug)]
pub struct Pathfinder {
    points: VecDeque<Position>,
    running: bool
}

impl Pathfinder {
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running
    }

    pub fn walk_path(&mut self, from: Position, mut path: Vec<Position>) {
        self.clear();

        path.insert(0, from);
        let chunks = path.chunks_exact(2);
        for chunk in chunks.clone() {
            self.calculate_path(chunk[0], chunk[1]);
        }

        let remainder = chunks.remainder();
        if remainder.is_empty() == false {
            let previous = match self.points.back() {
                Some(previous) => *previous,
                None => from
            };
            self.calculate_path(previous, remainder[0]);
        }
    }

    pub fn next_step(&mut self) -> Option<Position> {
        self.points.pop_front()
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.running = false;
    }

    fn calculate_path(&mut self, from: Position, to: Position) {
        if from.get_plane() != to.get_plane() {
            panic!("path could not be found; plane mismatch")
        }

        let mut delta_x = to.get_x() - from.get_x();
        let mut delta_y = to.get_y() - from.get_y();
        let max_steps = std::cmp::max(delta_x.abs(), delta_y.abs());
        for _ in 0..max_steps {
            match delta_x {
                x if x < 0 => delta_x += 1,
                x if x > 0 => delta_x -= 1,
                _ => {}
            }

            match delta_y {
                y if y < 0 => delta_y += 1,
                y if y > 0 => delta_y -= 1,
                _ => {}
            }

            self.points.push_back(Position::new(
                to.get_x() - delta_x,
                to.get_y() - delta_y,
            ));
        }
    }
}

impl Default for Pathfinder {
    fn default() -> Self {
        Self {
            points: VecDeque::with_capacity(16),
            running: false
        }
    }
}

impl Component for Pathfinder {
    type Storage = VecStorage<Self>;
}