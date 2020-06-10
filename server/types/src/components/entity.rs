use mithril_core::pos::Position;
use specs::{Component, VecStorage};
use std::collections::VecDeque;

use crate::CollisionDetector;
use pathfinding::prelude::{absdiff, astar};

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
    running: bool,
}

impl Pathfinder {
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running
    }

    pub fn walk_path(&mut self, detector: &CollisionDetector, from: Position, path: Vec<Position>) {
        if path.is_empty() {
            return;
        }
        
        self.clear();

        let mut full_route = VecDeque::new();
        let mut starting_point = from;
        for point in path {
            match astar(
                &starting_point,
                |&pos| {
                    let mut successors = Vec::with_capacity(8);
                    for y in -1..=1 {
                        for x in -1..=1 {
                            if x == 0 && y == 0 {
                                continue;
                            }
                            let successor = pos + (x, y);
                            if detector.is_traversable(successor) {
                                successors.push((successor, 1));
                            }
                        }
                    }
                    println!("Found {} successors to {:?}", successors.len(), pos);
                    successors.into_iter()
                },
                |&pos| absdiff(pos.get_x(), point.get_x()) + absdiff(pos.get_y(), point.get_y()),
                |&pos| pos == point,
            ) {
                Some((route, _steps)) => {
                    full_route.extend(route);
                }
                None => break,
            }

            starting_point = point;
        }

        self.points.append(&mut full_route);
    }

    pub fn next_step(&mut self) -> Option<Position> {
        self.points.pop_front()
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.running = false;
    }
}

impl Default for Pathfinder {
    fn default() -> Self {
        Self {
            points: VecDeque::with_capacity(16),
            running: false,
        }
    }
}

impl Component for Pathfinder {
    type Storage = VecStorage<Self>;
}
