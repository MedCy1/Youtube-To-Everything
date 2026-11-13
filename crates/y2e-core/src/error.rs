use thiserror::Error;

#[derive(Error, Debug)]
pub enum Y2eError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization / JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Process execution error: {command} exited with code {code:?}, stderr: {stderr}")]
    ProcessFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("{0}")]
    Install(String),

    #[error("Invalid time range: start ({start_seconds}s) must be strictly less than end ({end_seconds}s)")]
    InvalidTimeRange {
        start_seconds: u64,
        end_seconds: u64,
    },
}

pub type Result<T> = std::result::Result<T, Y2eError>;
