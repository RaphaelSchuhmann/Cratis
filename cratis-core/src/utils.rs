use std::path::Path;
use crate::error::{CratisError, CratisResult};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::{BufReader, Read};
use blake3::Hasher;
use notify::event::{EventKind};

/// Verifies that a given path exists and is a directory in the filesystem.
///
/// This function performs two checks:
/// 1. Verifies that the path exists
/// 2. Ensures the path points to a directory, not a file
///
/// # Arguments
/// * `path` - A reference to a Path object representing the path to check
///
/// # Returns
/// * `Ok(())` if the path exists and is a directory
/// * `Err(CratisError::InvalidPath)` if:
///     - The path does not exist
///     - The path exists but points to a file instead of a directory
///
/// # Examples
/// ```ignore
/// use std::path::Path;
///
/// // Check if a directory exists [[2]](https://stackoverflow.com/questions/32384594)
/// let dir_path = Path::new("/path/to/directory");
/// match ensure_path_exists(dir_path) {
///     Ok(()) => println!("Directory exists and is valid"),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
///
/// # Errors
/// Returns `CratisError::InvalidPath` with:
/// - The path string if the path doesn't exist
/// - A formatted message including the path if it points to a file
pub fn ensure_path_exists(path: &Path) -> CratisResult<()> {
    if !path.exists() {
        let path_str = path.to_string_lossy().into_owned();
        return Err(CratisError::InvalidPath(path_str));
    }

    if path.is_file() {
        let path_str = path.to_string_lossy().into_owned();
        return Err(CratisError::InvalidPath(format!("The path has to point to a folder: {}", path_str)));
    }

    Ok(())
}

/// Converts a byte size into a human-readable string with appropriate unit suffix.
///
/// Converts the given byte size to the largest applicable unit (B, KB, MB, etc.)
/// where the numeric value is less than 1024. The result is formatted with two
/// decimal places.
///
/// # Arguments
/// * `bytes` - The size in bytes as a f64
///
/// # Returns
/// A String containing the formatted size with appropriate unit suffix
///
/// # Examples
/// ```ignore
/// let size = to_human_readable_size(1234567.0);
/// assert_eq!(size, "1.18 MB");
///
/// let size = to_human_readable_size(500.0);
/// assert_eq!(size, "500.00 B");
/// ```
pub fn to_human_readable_size(bytes: f64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut size = bytes;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, units[unit_index])
}

/// Returns the current Unix timestamp in seconds since the Unix epoch.
/// 
/// # Returns
/// * `CratisResult<u64>` - The current Unix timestamp in seconds on success, or an error if the
///   system time cannot be retrieved.
/// 
/// # Errors
/// 
/// Returns `CratisError::Internal` if the system time cannot be obtained or is before the Unix epoch.
pub fn timestamp_now() -> CratisResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CratisError::Internal("Failed to get system time."))
        .map(|duration| duration.as_secs())
}

/// Sanitizes a filename by removing or replacing invalid characters.
/// 
/// This function removes control characters and replaces common invalid characters
/// with underscores. Invalid characters include: '/', '\', ':', '*', '?', '"', '<', '>', '|'
/// 
/// # Arguments
/// 
/// * `filename` - A string slice that holds the filename to sanitize
/// 
/// # Returns
/// * `String` - The sanitized filename. Returns "_" if the input filename becomes empty after sanitization.
/// 
/// # Examples
/// 
/// ```ignore
/// let safe_name = sanitize_filename("file:*.txt");
/// assert_eq!(safe_name, "file___.txt");
/// ```
pub fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

    let sanitized = filename.chars().map(|c| if invalid_chars.contains(&c) || c.is_control() {'_'} else { c }).collect::<String>();

    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    }
}

/// Calculates a hash of the contents of a file at the specified path.
/// 
/// Reads the file in chunks and calculates a hash using the configured hasher.
/// 
/// # Arguments
/// 
/// * `path` - A string slice containing the path to the file to hash
/// 
/// # Returns
/// 
/// * `CratisResult<String>` - THe hexadecimal string representation of the file's hash on success,
/// or an error if the file cannot be read.
/// 
/// # Errors
/// 
/// Returns `CratisError::IOError` if:
/// * The file cannot be opened
/// * An error occurs while reading the file
pub fn hash_file(path: &str) -> CratisResult<String> {
    let file = File::open(path).map_err(|e| CratisError::IoError(e.into()))?;
    let mut reader = BufReader::new(file);

    let mut hasher = Hasher::new();
    let mut buffer = [0u8; 1024];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| CratisError::IoError(e.into()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventAction {
    Create,
    Modify,
    Delete,
    Other
}

pub fn map_event_kinds(kind: &EventKind) -> EventAction {
    match kind {
        EventKind::Create(_) => EventAction::Create,
        EventKind::Modify(_) => EventAction::Modify,
        EventKind::Remove(_) => EventAction::Delete,
        _ => EventAction::Other
    }
}