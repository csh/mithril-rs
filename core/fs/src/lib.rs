use std::fs::File;
use std::io::{prelude::*, Cursor, SeekFrom};
use std::path::Path;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use crc32fast::Hasher;
use memmap::Mmap;

mod archive;
pub(crate) mod compression;
pub mod defs;

pub use archive::Archive;

const INDEX_SIZE: u64 = 6;
const CHUNK_SIZE: u64 = 512;
const HEADER_SIZE: u64 = 8;
const BLOCK_SIZE: u64 = HEADER_SIZE + CHUNK_SIZE;

#[derive(Debug)]
pub struct CacheFileSystem {
    data_file: Cursor<Mmap>,
    indices: Vec<CacheIndex>,
}

#[derive(Debug)]
struct CacheIndex(Cursor<Mmap>);

impl CacheIndex {
    fn get_block(&mut self, file_number: u64) -> anyhow::Result<(u64, u64)> {
        self.0.seek(SeekFrom::Start(INDEX_SIZE * file_number))?;
        let size = self.0.get_uint(3);
        let initial_block = self.0.get_uint(3);
        Ok((size, initial_block))
    }
}

impl CacheFileSystem {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let mut indices = Vec::with_capacity(5);

        for i in 0..5 {
            let index_path = path.join(format!("main_file_cache.idx{}", i));
            let idx: Cursor<Mmap> = File::open(&index_path)
                .and_then(|file| unsafe { memmap::Mmap::map(&file) })
                .map(Cursor::new)
                .unwrap_or_else(|_| panic!("Failed to map index file #{}", i));

            indices.push(CacheIndex(idx));
        }

        let cur: Cursor<Mmap> = File::open(path.join("main_file_cache.dat"))
            .and_then(|file| unsafe { memmap::Mmap::map(&file) })
            .map(Cursor::new)
            .expect("Failed to map data file");

        // TODO: Calculate CRC32 table prior to initialisation
        Ok(Self {
            data_file: cur,
            indices,
        })
    }

    pub fn len(&mut self, index_number: usize) -> anyhow::Result<usize> {
        let index = match self.indices.get_mut(index_number) {
            Some(index) => index,
            None => anyhow::bail!("Index not found"),
        };
        index.0.seek(SeekFrom::Start(0))?;
        Ok(index.0.remaining() / INDEX_SIZE as usize)
    }

    pub fn get_crc_table(&mut self) -> anyhow::Result<(Vec<u32>, u32)> {
        let num_archives = self.len(0)?;
        let hashes = (0..num_archives)
            .map(|index| {
                let buf = self.get_file(0, index as u64).unwrap_or_else(|_| {
                    panic!("Error reading file (0, {}) from the filesystem", index)
                });

                let mut hasher = Hasher::new();
                hasher.update(&buf[..]);
                let crc_hash = hasher.finalize();
                log::debug!("Archive {} CRC is {}", index, crc_hash);
                crc_hash
            })
            .collect::<Vec<u32>>();

        let archive_hash = hashes
            .iter()
            .fold(1234u32, |hash, crc| (hash << 1).wrapping_add(*crc));

        log::debug!("Archives hash is {}", archive_hash);
        Ok((hashes, archive_hash))
    }

    pub fn get_file(&mut self, index_number: usize, file_number: u64) -> anyhow::Result<Bytes> {
        let index = match self.indices.get_mut(index_number) {
            Some(index) => index,
            None => anyhow::bail!("Index not found"),
        };
        let (size, initial_block) = index.get_block(file_number)?;
        log::trace!(
            "Requested file (idx: {}, num: {}) starts at block {} and is {} bytes long",
            index_number,
            file_number,
            initial_block,
            size
        );

        let num_parts = if size % CHUNK_SIZE == 0 {
            size / CHUNK_SIZE
        } else {
            size / CHUNK_SIZE + 1
        };

        let mut position = initial_block * BLOCK_SIZE;
        let mut combined_buf = BytesMut::with_capacity(size as usize);

        for file_part in 0..num_parts {
            self.data_file.seek(SeekFrom::Start(position))?;
            let read_file_number = self.data_file.get_u16();
            let current_part = self.data_file.get_u16();
            let next_block = self.data_file.get_uint(3);
            let next_type = self.data_file.get_u8();

            assert_eq!(file_part, current_part as u64, "file part mismatch");
            let part_size = std::cmp::min(size as usize - combined_buf.len(), CHUNK_SIZE as usize);
            let mut part_buf = vec![0u8; part_size];
            assert_eq!(part_size, self.data_file.read(&mut part_buf)?);
            combined_buf.put(&part_buf[..]);
            position = next_block * BLOCK_SIZE;

            if size as usize > combined_buf.len() {
                assert_eq!(next_type, (index_number + 1) as u8);
                assert_eq!(read_file_number as u64, file_number);
            }
        }
        Ok(combined_buf.freeze())
    }

    pub fn get_archive(
        &mut self,
        index_number: usize,
        file_number: u64,
    ) -> anyhow::Result<Archive> {
        let contents = match self.get_file(index_number, file_number) {
            Ok(contents) => contents,
            Err(why) => return Err(why),
        };
        Archive::decode(contents)
    }
}

#[cfg(test)]
mod tests {
    /*
     * Due to people making modifications to caches and it being questionable to distribute
     * I'm not entirely sure we can run assertion based tests.
     *
     * It may be possible to run assertion based tests in a CI environment by downloading
     * a cache from a private URL passed as an environmental variable.
     *
     * For now tests should simply strive for loading cache data without error.
     */

    use super::*;

    fn open_filesystem() -> CacheFileSystem {
        CacheFileSystem::open("../../cache").expect("failed to open cache")
    }

    #[test]
    pub fn load_map_definitions() {
        if ci_info::is_ci() {
            return;
        }

        let mut cache = open_filesystem();
        let map_indices = defs::MapIndex::load(&mut cache).expect("map_indices");

        let map_data = map_indices.values().take(10).map(|index| {
            let map_file = defs::MapFile::load(&mut cache, index).expect("map_file");
            let map_objects = defs::MapObject::load(&mut cache, index).expect("map_objects");
            (map_file, map_objects)
        });
        dbg!(map_data.len());
    }

    #[test]
    pub fn load_entity_definitions() {
        if ci_info::is_ci() {
            return;
        }

        let mut cache = open_filesystem();
        let _ = defs::EntityDefinition::load(&mut cache).expect("entities");
    }

    #[test]
    pub fn load_item_definitions() {
        if ci_info::is_ci() {
            return;
        }

        let mut cache = open_filesystem();
        let items = defs::ItemDefinition::load(&mut cache).expect("items");
        items
            .iter()
            .take(50)
            .filter(|def| def.is_noted())
            .for_each(|item| {
                assert_eq!(item.is_stackable(), true, "all noted items are stackable");

                assert_ne!(
                    item.name(),
                    &String::default(),
                    "noted definitions should correctly copy information"
                );

                for idx in 0..5 {
                    assert!(
                        item.ground_action(idx).is_none() && item.inventory_action(idx).is_none(),
                        "noted items should not have any actions"
                    );
                }
            });
    }
}
