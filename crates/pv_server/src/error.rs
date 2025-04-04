use quinn::ConnectionError;
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("Connection ID error: {0}")]
    Error(#[from] Box<dyn Error + Send + Sync + 'static>),
    #[error("Io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),
}

