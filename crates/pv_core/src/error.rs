use bincode::error::{DecodeError, EncodeError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Decode error: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("Encode error: {0}")]
    EncodeError(#[from] EncodeError),
    #[error("Write error: {0}")]
    WriteError(#[from] quinn::WriteError),
    #[error("Read exact error: {0}")]
    ReadExactError(#[from] quinn::ReadExactError),
}
