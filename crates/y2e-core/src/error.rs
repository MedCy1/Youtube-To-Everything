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
    Install(InstallProblem),

    #[error("Invalid time range: start ({start_seconds}s) must be strictly less than end ({end_seconds}s)")]
    InvalidTimeRange {
        start_seconds: u64,
        end_seconds: u64,
    },
}

/// Why installing a tool failed. The interface words it in the user's language;
/// the English text here is for logs and for programs that print the error.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum InstallProblem {
    #[error("The data folder was not found.")]
    NoDataFolder,
    #[error("No {0} build for this system.")]
    NoBuild(String),
    #[error("curl or tar is missing on this system.")]
    MissingHelper,
    #[error("The ffmpeg download server did not answer.")]
    ServerSilent,
    #[error("Could not read {0}.")]
    Unreadable(String),
    #[error("Checksum for {0} not found.")]
    ChecksumMissing(String),
    #[error("The file {0} is corrupted and was rejected.")]
    Corrupted(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Y2eError>;
