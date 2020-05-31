/*
 * This example will decode map files from the cache and then use the "PNG" crate to
 * generate a 64x64 image showing whether a tile can be walked on or not.
 */

use std::fs::File;
use std::io::BufWriter;

use mithril_fs::*;

fn plane_to_rgba(plane: &defs::MapPlane) -> bytes::Bytes {
    use bytes::BufMut;
    let mut buf = bytes::BytesMut::new();
    for x in 0..64 {
        for z in 0..64 {
            let rgba: [u8; 4] = if plane.is_bridge(x, z) {
                [118, 94, 18, 255]
            } else if plane.is_walkable(x, z) {
                [30, 150, 50, 255]
            } else {
                [0, 0, 0, 0]
            };
            buf.put(&rgba[..]);
        }
    }
    buf.freeze()
}

fn main() {
    let cache_dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../cache"));

    let mut cache = CacheFileSystem::open(cache_dir).expect("cache");
    let map_indices = defs::MapIndex::load(&mut cache).expect("map_indices");

    if let Err(error) = std::fs::create_dir(cache_dir.join("map")) {
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create './cache/map' dir; {}", error);
        }
    }

    let indexed_planes = map_indices
        .values()
        .map(|index| {
            let map_file = defs::MapFile::load(&mut cache, index).expect("map_file");
            (index, map_file)
        })
        .collect::<Vec<_>>();

    indexed_planes.iter().for_each(|(index, map_file)| {
        let file_path =
            cache_dir
                .join("map")
                .join(format!("{}-{}.png", index.get_x(), index.get_y()));

        let w = File::create(file_path)
            .map(BufWriter::new)
            .expect("tile file");

        let mut encoder = png::Encoder::new(w, 64, 64);
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&plane_to_rgba(map_file.get_plane(0))[..])
            .expect("write_image_data");
    });
}
