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

/// Displays a Cratis error message to standard error (stderr).
///
/// # Arguments
///
/// * `error` - A reference to the CratisError to be displayed
/// * `debug` - A boolean flag that determines the error output format
///
/// When `debug` is true, displays the error using pretty-printed debug formatting ({:#?}).
/// When `debug` is false, displays a simple user-friendly error message using the Display trait.
///
/// # Examples
///
/// ```ignore
/// use cratis_core::CratisError;
///
/// let error = CratisError::InvalidInput("Invalid configuration");
/// display_error(&error, false); // Displays: "Invalid input provided: Invalid configuration"
/// display_error(&error, true);  // Displays detailed debug structure with formatting
/// ```
pub fn display_error(error: &CratisError, debug: bool) {
    if debug {
        eprintln!("Error (debug): {:#?}", error);
    } else {
        eprintln!("{error}");
    }
}
