#[derive(Debug, thiserror::Error)]
pub enum CratisError {
    #[error("Failed read/write file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to parse configuration: {0}")]
    ConfigParseError(#[from] serde_yaml::Error),

    #[error("Invalid input provided: {0}")]
    InvalidInput(&'static str),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

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

    #[error("Request error: {0}")]
    RequestError(&'static str),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Error generating token: {0}")]
    TokenError(String),

    #[error("Environment error: {0}")]
    EnvError(String),

    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CratisErrorLevel {
    // An info message for the user.
    Info,
    // An error occurred, but it is not fatal.
    Warning,
    // A fatal error occurred, exit program immediately!
    Fatal,
}

pub type CratisResult<T> = Result<T, CratisError>;

/// Displays a Cratis message to standard error (stderr).
///
/// # Arguments
///
/// * `error` - Optional reference to a CratisError to be displayed (used for Warning/Fatal).
/// * `level` - Info, Warning, or Fatal.
/// * `msg` - Optional message string for Info messages.
///
/// Behavior:
/// - Info: prints "Info: {msg}"
/// - Warning: prints "Warning: {error}"
/// - Fatal: prints "Fatal error: {error}" and exits with code 1.
///
/// # Examples
/// ```ignore
/// use cratis_core::error::{display_msg, CratisError, CratisErrorLevel};
///
/// display_msg(None, CratisErrorLevel::Info, Some("Starting backup".into()));
/// display_msg(Some(&CratisError::InvalidInput("Invalid configuration")), CratisErrorLevel::Warning, None);
/// ```
pub fn display_msg(error: Option<&CratisError>, level: CratisErrorLevel, msg: Option<String> /* msg is for info messages only */) {
    let error = error.unwrap_or(&CratisError::Unknown);
    let msg = msg.unwrap_or("".to_string());

    if level == CratisErrorLevel::Info {
        eprintln!("Info: {msg}");
    } else if level == CratisErrorLevel::Warning {
        eprintln!("Warning: {error}");
    } else if level == CratisErrorLevel::Fatal {
        eprintln!("Fatal error: {error}");
        std::process::exit(1);
    }
}
