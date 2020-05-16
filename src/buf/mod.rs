mod read;

pub use read::GameBuf;

#[derive(Debug, Clone, Copy)]
pub enum Transform {
    Add,
    Subtract,
    Negate
}