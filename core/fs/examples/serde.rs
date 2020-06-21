use mithril_fs::{defs, CacheError, CacheFileSystem};
use serde::Serialize;
use serde_json as json;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
enum WriteError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] json::Error),
}

fn main() {
    let cache_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../cache"));

    let mut cache = match CacheFileSystem::open(cache_dir) {
        Ok(cache) => cache,
        Err(CacheError::FileMapping { path, source, .. }) => {
            eprintln!("Failed to map required file {:?}", path);
            eprintln!("{}", source);
            return;
        }
        _ => unreachable!(),
    };

    let serde_dir = cache_dir.join("serde");
    if let Err(error) = fs::create_dir(&serde_dir) {
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create '/cache/serde' dir; {}", error);
        }
    }

    let items = defs::ItemDefinition::load(&mut cache).expect("items");
    let objects = defs::ObjectDefinition::load(&mut cache).expect("objects");
    let entities = defs::EntityDefinition::load(&mut cache).expect("entities");
    let map_data = defs::MapIndex::load(&mut cache).expect("map_indices");
    let map_data = map_data
        .iter()
        .map(|(_, index)| {
            (
                index,
                defs::MapObject::load(&mut cache, index).expect("map_objects"),
            )
        })
        .collect::<Vec<_>>();

    write_pretty(&serde_dir, "items.json", &items).expect("items.json");
    write_pretty(&serde_dir, "objects.json", &objects).expect("objects.json");
    write_pretty(&serde_dir, "entities.json", &entities).expect("entities.json");

    let map_dir = serde_dir.join("map_data");
    fs::create_dir(&map_dir).expect("map_data");
    for (index, objects) in map_data.iter() {
        let dir = map_dir.join(format!("{}-{}", index.get_x(), index.get_y()));
        fs::create_dir(&dir).expect("dir");
        write_pretty(&dir, "index.json", index).expect("index.json");
        write_pretty(&dir, "objects.json", objects).expect("index.json");
    }
}

fn write_pretty<T>(base_path: &PathBuf, file_name: &str, value: &T) -> Result<(), WriteError>
where
    T: Serialize,
{
    let serialized = json::to_string_pretty(value)?;
    fs::write(base_path.join(file_name), serialized)?;
    Ok(())
}
