use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};

use bytes::Buf;

use mithril_buf::GameBuf;

use crate::CacheFileSystem;

const MAP_PLANES: usize = 4;
const MAP_WIDTH: usize = 64;

const DEFAULT_TILE_HEIGHT: u16 = 304;
const FROM_LOWER_MULTIPLICAND: u16 = 8;
const PLANE_HEIGHT_DIFFERENCE: u16 = 240;
const MINIMUM_OVERLAY_TYPE: u8 = 49;
const ORIENTATION_COUNT: u8 = 4;
const MINIMUM_ATTRIBUTE_TYPE: u8 = 81;
const LOWEST_CONTINUED_TYPE: u8 = 2;

pub struct MapIndex {
    packed_coordinates: u16,
    map_file_id: u16,
    object_file_id: u16,
    member_only: bool,
}

impl MapIndex {
    pub fn load(cache: &mut CacheFileSystem) -> anyhow::Result<HashMap<u16, Self>> {
        let archive = cache.get_archive(0, 5)?;
        let mut buf = archive
            .get_entry("map_index")
            .map(|entry| entry.contents())
            .expect("map_index");

        let length = buf.len();
        let num_entries = length / (3 * std::mem::size_of::<u16>() + std::mem::size_of::<u8>());
        let mut definitions = HashMap::new();
        for _ in 0..num_entries {
            let coordinates = buf.get_u16();
            let terrain = buf.get_u16();
            let objects = buf.get_u16();
            let member_only = buf.get_u8() == 1;

            definitions.insert(
                coordinates,
                Self {
                    packed_coordinates: coordinates,
                    object_file_id: objects,
                    map_file_id: terrain,
                    member_only,
                },
            );
        }
        Ok(definitions)
    }

    pub fn get_x(&self) -> u16 {
        (self.packed_coordinates >> 8 & 0xFF) * MAP_WIDTH as u16
    }

    pub fn get_y(&self) -> u16 {
        (self.packed_coordinates & 0xFF) * MAP_WIDTH as u16
    }

    pub fn is_member_only(&self) -> bool {
        self.member_only
    }
}

impl Debug for MapIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut ds = f.debug_struct("MapIndex");
        ds.field("x", &self.get_x());
        ds.field("y", &self.get_y());
        ds.field("object_file_id", &self.object_file_id);
        ds.field("map_file_id", &self.map_file_id);
        ds.field("member_only", &self.member_only);
        ds.finish()
    }
}

pub struct MapObject {
    id: u16,
    variant: u16,
    orientation: u8,
    packed_coordinates: i16,
}

impl MapObject {
    pub fn load(cache: &mut CacheFileSystem, index: &MapIndex) -> anyhow::Result<Vec<Self>> {
        let buf = cache.get_file(4, index.object_file_id as u64)?;
        let mut buf = crate::compression::decompress_gzip(buf)?;

        let mut objects = Vec::new();

        let mut id = -1;
        let mut id_offset = buf.get_smart() as i32;
        while id_offset != 0 {
            id += id_offset as i32;

            let mut packed = 0;
            let mut position_offset = buf.get_smart() as i16;
            while position_offset != 0 {
                packed += position_offset - 1;

                let attributes = buf.get_u8();
                let ty = attributes >> 2;
                let orientation = attributes & 0x3;
                objects.push(MapObject {
                    id: id as u16,
                    variant: ty as u16,
                    orientation: orientation as u8,
                    packed_coordinates: packed,
                });
                position_offset = buf.get_smart() as i16;
            }

            id_offset = buf.get_smart() as i32;
        }

        Ok(objects)
    }

    pub fn get_x(&self) -> i16 {
        (self.packed_coordinates >> 6) & 0x3F
    }

    pub fn get_y(&self) -> i16 {
        self.packed_coordinates & 0x3F
    }

    pub fn get_plane(&self) -> i16 {
        (self.packed_coordinates >> 12) & 0x3
    }
}

impl Debug for MapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("MapObject");
        ds.field("id", &self.id);
        ds.field("variant", &self.variant);
        ds.field("x", &self.get_x());
        ds.field("z", &self.get_y());
        ds.field("plane", &self.get_plane());
        ds.field("orientation", &self.orientation);
        ds.finish()
    }
}

pub struct MapPlane {
    tiles: [[Tile; MAP_WIDTH]; MAP_WIDTH],
}

impl MapPlane {
    pub fn get_tile(&self, x: usize, z: usize) -> &Tile {
        assert!(x < MAP_WIDTH, "x out of bounds; {}", x);
        assert!(z < MAP_WIDTH, "z out of bounds; {}", z);
        &self.tiles[x][z]
    }

    pub fn is_walkable(&self, x: usize, z: usize) -> bool {
        self.get_tile(x, z).is_walkable()
    }

    pub fn is_bridge(&self, x: usize, z: usize) -> bool {
        self.get_tile(x, z).is_bridge()
    }

    pub fn height(&self, x: usize, z: usize) -> u16 {
        self.get_tile(x, z).height
    }
}

impl Default for MapPlane {
    fn default() -> Self {
        Self {
            tiles: [[Tile::default(); MAP_WIDTH]; MAP_WIDTH],
        }
    }
}

pub struct MapFile {
    planes: Vec<MapPlane>,
}

impl MapFile {
    pub fn load(cache: &mut CacheFileSystem, index: &MapIndex) -> anyhow::Result<Self> {
        let file = cache
            .get_file(4, index.map_file_id as u64)
            .expect("failed to read map_file_id");

        let mut buf = crate::compression::decompress_gzip(file).expect("gzip decode failed");
        let mut map_file = MapFile::new();
        for plane in 0..MAP_PLANES {
            for x in 0..MAP_WIDTH {
                for z in 0..MAP_WIDTH {
                    let mut tile = map_file.planes[plane].tiles[x][z];
                    let mut read = LOWEST_CONTINUED_TYPE;
                    // TODO: Optimise. ASAP.
                    while read >= LOWEST_CONTINUED_TYPE {
                        read = buf.get_u8();
                        match TileUpdate::from(read) {
                            TileUpdate::Height => {
                                tile.height = if plane == 0 {
                                    DEFAULT_TILE_HEIGHT
                                } else {
                                    map_file.planes[plane].tiles[x][z].height
                                        + PLANE_HEIGHT_DIFFERENCE
                                }
                            }
                            TileUpdate::HeightFromLower => {
                                let height = buf.get_u8();
                                let below = if plane == 0 {
                                    0
                                } else {
                                    map_file.planes[plane].tiles[x][z].height
                                };

                                tile.height = if height == 1 { 0 } else { height as u16 }
                                    * FROM_LOWER_MULTIPLICAND
                                    + below as u16;
                            }
                            TileUpdate::Overlay(tile_type) => {
                                tile.overlay = buf.get_u8();
                                tile.overlay_type =
                                    (tile_type - LOWEST_CONTINUED_TYPE) / ORIENTATION_COUNT;
                                tile.overlay_orientation =
                                    (tile_type - LOWEST_CONTINUED_TYPE) % ORIENTATION_COUNT;
                            }
                            TileUpdate::Attributes(tile_type) => {
                                tile.attributes = tile_type - MINIMUM_OVERLAY_TYPE
                            }
                            TileUpdate::Underlay(tile_type) => {
                                tile.underlay = tile_type - MINIMUM_ATTRIBUTE_TYPE
                            }
                        };
                    }
                    map_file.planes[plane].tiles[x][z] = tile;
                }
            }
        }
        Ok(map_file)
    }

    fn new() -> MapFile {
        Self {
            planes: vec![
                MapPlane::default(),
                MapPlane::default(),
                MapPlane::default(),
                MapPlane::default(),
            ],
        }
    }

    pub fn get_plane(&self, plane: usize) -> &MapPlane {
        assert!(plane < MAP_PLANES, "plane out of bounds; {}", plane);
        &self.planes[plane]
    }

    pub fn get_tile(&self, plane: usize, x: usize, z: usize) -> &Tile {
        self.get_plane(plane).get_tile(x, z)
    }

    pub fn is_walkable(&self, plane: usize, x: usize, z: usize) -> bool {
        self.get_tile(plane, x, z).is_walkable()
    }

    pub fn is_bridge(&self, plane: usize, x: usize, z: usize) -> bool {
        self.get_tile(plane, x, z).is_bridge()
    }

    pub fn height(&self, plane: usize, x: usize, z: usize) -> u16 {
        self.get_tile(plane, x, z).height
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Tile {
    height: u16,
    overlay: u8,
    overlay_type: u8,
    overlay_orientation: u8,
    underlay: u8,
    attributes: u8,
}

impl Tile {
    pub fn is_walkable(self) -> bool {
        self.attributes & 0x1 != 0x1
    }

    pub fn is_bridge(self) -> bool {
        self.attributes & 0x2 == 0x2
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            height: DEFAULT_TILE_HEIGHT,
            overlay: 0,
            overlay_type: 0,
            overlay_orientation: 0,
            underlay: 0,
            attributes: 0,
        }
    }
}

enum TileUpdate {
    Height,
    HeightFromLower,
    Overlay(u8),
    Attributes(u8),
    Underlay(u8),
}

impl From<u8> for TileUpdate {
    fn from(b: u8) -> Self {
        if b == 0 {
            Self::Height
        } else if b == 1 {
            Self::HeightFromLower
        } else if b <= 49 {
            Self::Overlay(b)
        } else if b <= 81 {
            Self::Attributes(b)
        } else {
            Self::Underlay(b)
        }
    }
}
