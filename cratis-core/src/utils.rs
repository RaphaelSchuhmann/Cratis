use std::path::Path;
use crate::error::{CratisError, CratisResult};

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