use std::fs::File;
use std::io::{prelude::*, Cursor, SeekFrom};
use std::path::Path;

use bytes::{Buf, Bytes, BytesMut};
use crc32fast::Hasher;
use memmap::Mmap;

mod archive;
mod error;

pub(crate) mod compression;
pub mod defs;

pub use archive::Archive;
pub use error::{ArchiveError, CacheError, FilePartError};

const INDEX_SIZE: u64 = 6;
const CHUNK_SIZE: u64 = 512;
const HEADER_SIZE: u64 = 8;
const BLOCK_SIZE: u64 = HEADER_SIZE + CHUNK_SIZE;

pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(Debug)]
pub struct CacheFileSystem {
    data_file: Mmap,
    indices: Vec<CacheIndex>,
}

#[derive(Debug)]
struct CacheIndex {
    index_file: Mmap,
    len: usize,
}

impl CacheIndex {
    fn get_block(&self, file_number: usize) -> Result<(u64, u64)> {
        let mut cursor = Cursor::new(&self.index_file);
        cursor.seek(SeekFrom::Start(INDEX_SIZE * file_number as u64))?;
        let size = cursor.get_uint(3);
        let initial_block = cursor.get_uint(3);
        Ok((size, initial_block))
    }
}

impl CacheFileSystem {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut indices = Vec::with_capacity(5);

        for i in 0..5 {
            let path = path.join(format!("main_file_cache.idx{}", i));
            match File::open(&path).and_then(|file| unsafe { memmap::Mmap::map(&file) }) {
                Ok(file) => indices.push(CacheIndex {
                    len: file.len() / INDEX_SIZE as usize,
                    index_file: file,
                }),
                Err(source) => return Err(CacheError::FileMapping { path, source }),
            };
        }

        let path = path.join("main_file_cache.dat");
        let cur = match File::open(&path).and_then(|file| unsafe { memmap::Mmap::map(&file) }) {
            Ok(file) => file,
            Err(source) => return Err(CacheError::FileMapping { path, source }),
        };

        // TODO: Calculate CRC32 table prior to initialisation
        Ok(Self {
            data_file: cur,
            indices,
        })
    }

    pub fn len(&self, index_number: usize) -> Result<usize> {
        let index = self
            .indices
            .get(index_number)
            .ok_or(CacheError::IndexNotFound(index_number))?;
        Ok(index.len)
    }

    pub fn get_crc_table(&mut self) -> Result<(Vec<u32>, u32)> {
        let num_archives = self.len(0)?;
        let mut hashes = vec![0; num_archives];
        for (index, hash) in hashes.iter_mut().enumerate() {
            let buf = self.get_file(0, index)?;

            let mut hasher = Hasher::new();
            hasher.update(&buf[..]);
            *hash = hasher.finalize();
            log::debug!("Archive {} CRC is {}", index, hash);
        }

        let archive_hash = hashes
            .iter()
            .fold(1234u32, |hash, crc| (hash << 1).wrapping_add(*crc));

        log::debug!("Archives hash is {}", archive_hash);
        Ok((hashes, archive_hash))
    }

    pub fn get_file(&self, index_number: usize, file_number: usize) -> Result<Bytes> {
        let index = self
            .indices
            .get(index_number)
            .ok_or(CacheError::IndexNotFound(index_number))?;

        if file_number > index.len {
            return Err(CacheError::FileNotFound(index_number, file_number));
        }

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
        } as u16;

        let mut position = initial_block * BLOCK_SIZE;
        let mut combined_buf = BytesMut::with_capacity(size as usize);

        let mut cursor = Cursor::new(&self.data_file);
        for file_part in 0..num_parts {
            cursor.seek(SeekFrom::Start(position))?;

            let read_file_number = cursor.get_u16();
            let read_file_part = cursor.get_u16();
            let next_block = cursor.get_uint(3);
            let next_type = cursor.get_u8();

            if file_part != read_file_part {
                return Err(FilePartError::PartMismatch {
                    expected: file_part,
                    actual: read_file_part,
                }
                .into());
            }

            let part_size = std::cmp::min(size as usize - combined_buf.len(), CHUNK_SIZE as usize);
            let mut part_buf = vec![0; part_size];

            let read = cursor.read(&mut part_buf)?;

            if read != part_size {
                return Err(FilePartError::Length {
                    expected: part_size,
                    actual: read,
                }
                .into());
            }

            combined_buf.extend_from_slice(&part_buf);
            position = next_block * BLOCK_SIZE;

            if size as usize > combined_buf.len() {
                assert_eq!(next_type, (index_number + 1) as u8);
                assert_eq!(read_file_number as usize, file_number);
            }
        }
        Ok(combined_buf.freeze())
    }

    pub fn get_archive(&self, index_number: usize, file_number: usize) -> Result<Archive> {
        let contents = self.get_file(index_number, file_number)?;
        Archive::decode(contents)
    }
}


#[cfg(feature = "serde")]
pub(crate) fn skip_empty_options<T>(options: &[Option<T>]) -> bool {
    options.iter().all(Option::is_none)
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

    macro_rules! skip_ci {
        () => {
            if ci_info::is_ci() {
                return;
            }
        };
    }

    fn open_filesystem() -> CacheFileSystem {
        let cache_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../cache");
        CacheFileSystem::open(cache_path).expect("failed to open cache")
    }

    #[test]
    pub fn load_map_definitions() {
        skip_ci!();

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
        skip_ci!();

        let mut cache = open_filesystem();
        let _ = defs::EntityDefinition::load(&mut cache).expect("entities");
    }

    #[test]
    pub fn load_object_definitions() {
        skip_ci!();

        let mut cache = open_filesystem();
        let _ = defs::ObjectDefinition::load(&mut cache).expect("objects");
    }

    #[test]
    pub fn load_item_definitions() {
        skip_ci!();

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

    #[test]
    pub fn error_file_mapping() {
        match CacheFileSystem::open("invalid").err() {
            Some(CacheError::FileMapping { .. }) => {}
            _ => panic!("should fail with FileMapping error"),
        }
    }

    #[test]
    pub fn error_invalid_index() {
        skip_ci!();

        let cache = open_filesystem();
        match cache.get_file(5, 0) {
            Err(CacheError::IndexNotFound(_)) => {}
            _ => panic!("Revision only has 5 index files"),
        }

        match cache.get_file(0, 100) {
            Err(CacheError::FileNotFound(_, _)) => {}
            _ => panic!("Index 0 does not contain 100 files"),
        }
    }
}
