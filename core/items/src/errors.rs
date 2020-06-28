use thiserror::Error;

#[derive(Error, Debug)]
pub enum ItemLookupError {
    #[error("could not find an item with ID {0}")]
    NotFound(u16),
    #[error("the item with ID {0} is not stackable")]
    NotStackable(u16),
    #[error("expected index < 5 but got {0}")]
    IndexOutOfBounds(usize),
    #[error(transparent)]
    Cache(#[from] mithril_fs::CacheError),
}
