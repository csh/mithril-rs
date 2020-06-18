use thiserror::Error;
use nom::Err;

#[derive(Error, Debug)]
pub enum JaggrabError {
    #[error("JAGGRAB request was malformed or incomplete")]
    InvalidRequest,
    #[error("file '{0}' was requested but could not be found")]
    FileNotFound(String),
    #[error(transparent)]
    InvalidFileName(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl<I> From<Err<I>> for JaggrabError where I: Sized {
    fn from(_: Err<I>) -> Self {
        JaggrabError::InvalidRequest
    }
}