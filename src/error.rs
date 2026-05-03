use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Audio error: {0}")]
    Audio(String),
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Playlist error: {0}")]
    Playlist(String),
    #[error("No tracks loaded")]
    NoTracks,
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
