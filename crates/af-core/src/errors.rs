//! Stable error types and codes for CLI/JSON protocol.

use serde::Serialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Error)]
pub enum AfError {
    #[error("{message}")]
    General { code: String, message: String, path: Option<String> },
    #[error("{message}")]
    InvalidInput { code: String, message: String, path: Option<String> },
    #[error("{message}")]
    Io { code: String, message: String, path: Option<String> },
    #[error("{message}")]
    Encoding { code: String, message: String, path: Option<String> },
    #[error("{message}")]
    Conflict { code: String, message: String, path: Option<String> },
    #[error("{message}")]
    MatchCount { code: String, message: String, path: Option<String> },
    #[error("{message}")]
    Safety { code: String, message: String, path: Option<String> },
}

impl AfError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AfError::General { .. } => 1,
            AfError::InvalidInput { .. } => 2,
            AfError::Io { .. } => 3,
            AfError::Encoding { .. } => 4,
            AfError::Conflict { .. } => 5,
            AfError::MatchCount { .. } => 6,
            AfError::Safety { .. } => 7,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AfError::General { code, .. }
            | AfError::InvalidInput { code, .. }
            | AfError::Io { code, .. }
            | AfError::Encoding { code, .. }
            | AfError::Conflict { code, .. }
            | AfError::MatchCount { code, .. }
            | AfError::Safety { code, .. } => code,
        }
    }

    pub fn path(&self) -> Option<&str> {
        match self {
            AfError::General { path, .. }
            | AfError::InvalidInput { path, .. }
            | AfError::Io { path, .. }
            | AfError::Encoding { path, .. }
            | AfError::Conflict { path, .. }
            | AfError::MatchCount { path, .. }
            | AfError::Safety { path, .. } => path.as_deref(),
        }
    }

    pub fn payload(&self) -> ErrorPayload {
        ErrorPayload {
            code: self.code().to_string(),
            message: self.to_string(),
            path: self.path().map(|s| s.to_string()),
        }
    }

    pub fn general(msg: impl Into<String>) -> Self {
        Self::General { code: "GENERAL_ERROR".into(), message: msg.into(), path: None }
    }
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput { code: "INVALID_INPUT".into(), message: msg.into(), path: None }
    }
    pub fn io(path: &Path, err: impl std::fmt::Display) -> Self {
        let msg = err.to_string();
        let code = if msg.to_ascii_lowercase().contains("not found")
            || msg.to_ascii_lowercase().contains("cannot find")
            || msg.to_ascii_lowercase().contains("no such file")
        {
            "FILE_NOT_FOUND"
        } else if msg.to_ascii_lowercase().contains("permission")
            || msg.to_ascii_lowercase().contains("access is denied")
        {
            "PERMISSION_DENIED"
        } else {
            "IO_ERROR"
        };
        Self::Io { code: code.into(), message: format!("{}: {}", path.display(), msg), path: Some(path.display().to_string()) }
    }
    pub fn io_msg(msg: impl Into<String>) -> Self {
        Self::Io { code: "IO_ERROR".into(), message: msg.into(), path: None }
    }
    pub fn encoding(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Encoding { code: code.into(), message: msg.into(), path: None }
    }
    pub fn encoding_path(code: impl Into<String>, path: &Path, msg: impl Into<String>) -> Self {
        Self::Encoding { code: code.into(), message: msg.into(), path: Some(path.display().to_string()) }
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict { code: "HASH_CONFLICT".into(), message: msg.into(), path: None }
    }
    pub fn match_count(msg: impl Into<String>) -> Self {
        Self::MatchCount { code: "MATCH_COUNT_MISMATCH".into(), message: msg.into(), path: None }
    }
    pub fn safety(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Safety { code: code.into(), message: msg.into(), path: None }
    }
}
