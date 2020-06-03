use ahash::AHashMap;
use mithril_core::fs::{defs, CacheFileSystem};
use mithril_core::pos::Position;
use pathfinding::grid::Grid;

#[derive(Debug)]
pub struct CollisionDetector {
    impassable: AHashMap<(u8, i16, i16), Grid>,
}

impl CollisionDetector {
    pub fn new(cache: &mut CacheFileSystem) -> anyhow::Result<Self> {
        let map_indices = defs::MapIndex::load(cache)?;
        let mut impassable = AHashMap::new();
        for index in map_indices.values() {
            let map_file = defs::MapFile::load(cache, index).expect("map file");
            for plane in 0..4 {
                let mut grid = Grid::new(64, 64);
                grid.enable_diagonal_mode();
                grid.add_borders();
                grid.fill();

                let pos = Position::default();
                let (local_x, local_z) = pos.get_relative(pos);
                for x in 0..64 {
                    for z in 0..64 {
                        if !map_file.is_walkable(plane, x, z) || map_file.is_bridge(plane, x, z) {
                            grid.remove_vertex(&(x, z));
                        }
                    }
                }

                let key = (
                    plane as u8,
                    index.get_x() as i16 / 64i16,
                    index.get_y() as i16 / 64i16,
                );
                impassable.insert(key, grid);
            }
        }

        Ok(Self { impassable })
    }

    pub fn is_traversable(&self, pos: Position) -> bool {
        let search_x = pos.get_x() / 64;
        let search_y = pos.get_y() / 64;
        match self.impassable.get(&(pos.get_plane(), search_x, search_y)) {
            Some(grid) => {
                let (local_x, local_y) = pos.get_relative(pos);
                return grid.has_vertex(&(local_x as _, local_y as _));
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::collision_detection::CollisionDetector;
    use mithril_core::fs::CacheFileSystem;
    use mithril_core::pos::Position;

    #[test]
    pub fn test_collisions() {
        use pathfinding::prelude::{absdiff, astar, Grid};

        let mut cache = CacheFileSystem::open("../../cache").expect("cache");
        let detector = CollisionDetector::new(&mut cache).expect("detector");

        println!("Map coordinates loaded, beginning pathfinding");

        let start = Position::default();
        let goal = Position::new(start.get_x() + 2, start.get_y() + 4);
        assert!(
            detector.is_traversable(start),
            "start is non-traversable; this is a bug"
        );
        assert!(detector.is_traversable(goal), "goal is non-traversable");

        let result = astar(
            &start,
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
            |&pos| absdiff(pos.get_x(), goal.get_x()) + absdiff(pos.get_y(), goal.get_y()),
            |&pos| pos.get_x() == goal.get_x() && pos.get_y() == goal.get_y(),
        );

        match result {
            Some((path, cost)) => {
                dbg!(cost);
                dbg!(path);
            }
            None => {
                println!("No path found between {:?} and {:?}", start, goal);
            }
        }
    }
}
