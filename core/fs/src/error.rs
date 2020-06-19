use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("failed to open {path}; {source}")]
    FileMapping { path: PathBuf, source: io::Error },
    #[error("index {0} could not be found")]
    IndexNotFound(usize),
    #[error("file {1} was not found in index {0}")]
    FileNotFound(usize, usize),
    #[error(transparent)]
    FilePart(#[from] FilePartError),
    #[error(transparent)]
    Archive(#[from] ArchiveError),
    #[error("unexpected opcode when decoding a {ty} definition: {:02X}", opcode)]
    DecodeDefinition { ty: &'static str, opcode: u8 },
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum FilePartError {
    #[error("expected file part '{expected}' but read part '{actual}'")]
    PartMismatch { expected: u16, actual: u16 },
    #[error("expected {expected} bytes but read {actual}")]
    Length { expected: usize, actual: usize },
}

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("expected archive to be {expected} bytes but decompressed {actual} bytes")]
    LengthMismatch { expected: usize, actual: usize },
    #[error("archive did not contain an entry named '{0}'")]
    EntryNotFound(&'static str),
}
