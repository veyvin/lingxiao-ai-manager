//! Application error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Cursor data directory was not found")]
    CursorDataNotFound,

    #[error("Failed to read storage.json: {0}")]
    StorageJsonError(String),

    #[error("Database read failed: {0}")]
    DatabaseError(String),

    #[error("Network request failed: {0}")]
    NetworkError(String),

    #[error("JSON parsing failed: {0}")]
    JsonParseError(String),

    #[error("Registry operation failed: {0}")]
    RegistryError(String),

    #[error("Local session is invalid or expired")]
    InvalidToken,

    #[error("No logged-in Cursor account was found")]
    NotLoggedIn,

    #[error("This operation requires administrator permissions")]
    AdminRequired,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Unknown(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::JsonParseError(e.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::NetworkError(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
