use thiserror::Error;

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