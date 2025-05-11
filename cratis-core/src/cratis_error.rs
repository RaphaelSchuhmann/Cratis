#[derive(Debug, thiserror::Error)]
pub enum CratisError {
    #[error("Failed read/write file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(&'static str),

    #[error("Failed to parse configuration: {0}")]
    ConfigParseError(#[from] serde_yaml::Error),

    #[error("Invalid input provided: {0}")]
    InvalidInput(&'static str),

    #[error("Network or connection error: {0}")]
    ConnectionIssue(&'static str),

    #[error("Authentication failed: {0}")]
    AuthFailure(&'static str),

    #[error("Operation timed out")]
    Timeout,

    #[error("Backup process failed: {0}")]
    BackupFailure(&'static str),

    #[error("Unsupported operation: {0}")]
    Unsupported(&'static str),

    #[error("Internal error: {0}")]
    Internal(&'static str),

    #[error("Unknown error")]
    Unknown,
}

pub type CratisResult<T> = Result<T, CratisError>;

pub fn display_error(error: &CratisError, debug: bool) {
    if debug {
        eprintln!("{error}");
    } else {
        eprintln!("{error}");
    }
}
