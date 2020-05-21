pub use read::GameBuf;
pub use write::GameBufMut;
pub use write::BitWriter;

mod read;
mod write;

#[derive(Debug, Clone, Copy)]
pub enum Transform {
    Add,
    Subtract,
    Negate,
}
