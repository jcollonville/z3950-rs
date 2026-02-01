use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("ber encode: {0}")]
    BerEncode(String),
    #[error("ber decode: {0}")]
    BerDecode(String),
    #[error("invalid object identifier: {0}")]
    InvalidOid(String),
    #[error("invalid visible string: {0}")]
    InvalidVisibleString(String),
    #[error("marc parse: {0}")]
    Marc(String),
    #[error("protocol: {0}")]
    Protocol(String),
    #[error("received frame exceeds maximum size ({max} bytes)")]
    FrameTooLarge { max: usize },
}


