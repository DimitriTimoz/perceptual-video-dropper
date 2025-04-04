use bincode::error::{self, DecodeError, EncodeError};
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
    #[error("Encode error: {0}")]
    EncodeError(#[from] EncodeError),
    #[error("Decode error: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("Write error: {0}")]
    WriteError(#[from] quinn::WriteError),
    #[error("Read exact error: {0}")]
    ReadExactError(#[from] quinn::ReadExactError),
}
