mod read;
mod write;

pub use read::GameBuf;
pub use write::GameBufMut;

#[derive(Debug, Clone, Copy)]
pub enum Transform {
    Add,
    Subtract,
    Negate,
}
