pub use read::GameBuf;
pub use write::BitWriter;
pub use write::GameBufMut;

mod read;
mod write;

#[derive(Debug, Clone, Copy)]
pub enum Transform {
    Add,
    Subtract,
    Negate,
}
