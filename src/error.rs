use thiserror::Error;

#[derive(Error, Debug)]
pub enum BeatPartnerError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Audio analysis error: {0}")]
    AudioAnalysis(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Real-time safety violation: {0}")]
    RealtimeViolation(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BeatPartnerError>;
