//! Stable error types and codes for CLI/JSON protocol.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AfError {
    #[error("general failure: {0}")]
    General(String),
    #[error("invalid arguments or input: {0}")]
    InvalidInput(String),
    #[error("file not found or inaccessible: {0}")]
    Io(String),
    #[error("encoding or binary file issue: {0}")]
    Encoding(String),
    #[error("concurrent modification conflict: {0}")]
    Conflict(String),
    #[error("match count does not meet expectation: {0}")]
    MatchCount(String),
    #[error("operation rejected by safety policy: {0}")]
    Safety(String),
}

impl AfError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AfError::General(_) => 1,
            AfError::InvalidInput(_) => 2,
            AfError::Io(_) => 3,
            AfError::Encoding(_) => 4,
            AfError::Conflict(_) => 5,
            AfError::MatchCount(_) => 6,
            AfError::Safety(_) => 7,
        }
    }
}
