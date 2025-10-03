use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use blake3::Hasher;
use crate::error::{display_msg, CratisError, CratisResult, CratisErrorLevel};
use crate::config::{CratisConfig};
use glob::Pattern;
use rand::distr::{Alphanumeric, SampleString};
use rand::Rng;

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

/// Checks if a path matches any of the provided exclusion patterns.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` to check
/// * `exclude_patterns` - A slice of `Pattern`s to match against
///
/// # Returns
///
/// Returns `true` if the path matches any of the exclusion patterns,
/// `false` otherwise.
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
///
/// let patterns = vec![Pattern::new("*.log"), Pattern::new("target/*")];
/// let path = Path::new("app.log");
/// assert!(is_excluded(&path, &patterns));
///
/// let source_file = Path::new("src/main.rs");
/// assert!(!is_excluded(&source_file, &patterns));
/// ```
///
/// # Implementation Details
///
/// Uses the `Iterator::any()` method to check if any pattern matches the given path,
/// providing short-circuit evaluation for efficiency.
pub fn is_excluded(path: &Path, exclude_patterns: &[Pattern]) -> bool {
    exclude_patterns.iter().any(|pattern| pattern.matches_path(path))
}

/// Checks if a path points to a file.
///
/// # Arguments
///
/// * `dir` - A string slice containing the path to check
///
/// # Returns
///
/// * `bool` - `true` if the path exists and is a file, `false` otherwise
///
/// # Examples
///
/// ```ignore
/// if is_path_file("/path/to/file.txt") {
///     println!("This is a file");
/// } else {
///     println!("This is not a file or doesn't exist");
/// }
/// ```
pub fn is_path_file(dir: &str) -> bool {
    let path = Path::new(dir);

    match fs::metadata(path) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false
    }
}

/// Recursively collects all files in a directory, respecting exclusion patterns.
///
/// This function traverses the specified directory and all its subdirectories,
/// collecting paths to all files while applying exclusion patterns from the
/// application configuration.
///
/// # Arguments
///
/// * `dir` - A reference to a String containing the directory path to scan
///
/// # Returns
///
/// * `CratisResult<Vec<PathBuf>>` - A vector of PathBuf objects representing all files
///   found in the directory tree, or an error if the directory cannot be accessed
///
/// # Errors
///
/// Returns `CratisError::InvalidPath` if:
/// * The path points to a file instead of a directory
/// * The path does not exist
///
/// May also return filesystem-related errors during directory traversal.
///
/// # Examples
///
/// ```ignore
/// match get_files_in_directory(&String::from("/path/to/directory")) {
///     Ok(files) => {
///         println!("Found {} files", files.len());
///         for file in files {
///             println!("{}", file.display());
///         }
///     },
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
pub fn get_files_in_directory(dir: &String) -> CratisResult<Vec<PathBuf>> {
    // Check if directory is a file (Just in case)
    if is_path_file(&dir) {
        // Warning
        return Err(CratisError::InvalidPath(format!("The path has to point to a folder: {}", dir)));
    }

    // Check if directory exists
    if !Path::new(&dir).exists() {
        // Warning
        return Err(CratisError::InvalidPath(format!("The path does not exist: {}", dir)));
    }

    let config: &CratisConfig = crate::config::get_config_cli();

    let exclude_dirs: &Vec<String> = &config.backup.exclude.clone().unwrap_or_default();

    let mut exclude_patterns: Vec<Pattern> = Vec::new();

    if !exclude_dirs.is_empty() {
        for pattern in exclude_dirs.iter() {
            match Pattern::new(pattern) {
                Ok(p) => exclude_patterns.push(p),
                Err(e) => display_msg(Some(&CratisError::ConfigError(format!("Invalid exclusion pattern '{}': {}", pattern, e))), CratisErrorLevel::Fatal, None)
            }
        }
    }

    let path = Path::new(&dir);
    let mut file_paths: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if is_excluded(&path, &exclude_patterns) { continue; }

        if path.is_dir() {
            let sub_dir_files = get_files_in_directory(&path.to_str().unwrap().to_string())?;
            file_paths.extend(sub_dir_files);
        } else {
            file_paths.push(path);
        }
    }

    Ok(file_paths)
}

/// Opens a file at the specified path with enhanced error handling.
///
/// Attempts to open the file and provides specific error handling for common issues.
/// Converts "file not found" errors to a more descriptive `InvalidPath` error.
///
/// # Arguments
///
/// * `file_path` - A PathBuf containing the path to the file to open
///
/// # Returns
///
/// * `CratisResult<File>` - A file handle on success
///
/// # Errors
///
/// * `CratisError::InvalidPath` - If the file does not exist
/// * `CratisError::IoError` - For other I/O errors (permissions, etc.)
///
/// # Examples
///
/// ```ignore
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("/path/to/file.txt");
/// match load_file(path) {
///     Ok(file) => println!("File opened successfully"),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
pub fn load_file(file_path: PathBuf) -> CratisResult<(File, String, Option<String>)> {
    let file = File::open(&file_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            // Warning
            CratisError::InvalidPath(format!("File not found: {}", &file_path.to_str().unwrap().to_string()))
        } else {
            CratisError::IoError(e.into())
        }
    })?;

    Ok((file, get_file_name(file_path.clone()), Some(file_path.to_str().unwrap().to_string())))
}

/// Extracts the filename from a path.
///
/// This function attempts to extract the filename component from a path.
/// If the filename cannot be extracted or converted to a string, it returns "unknown_file".
///
/// # Arguments
///
/// * `file_path` - A PathBuf containing the path from which to extract the filename
///
/// # Returns
///
/// * `String` - The extracted filename as a String, or "unknown_file" if extraction fails
///
/// # Examples
///
/// ```ignore
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("/path/to/document.txt");
/// let filename = get_file_name(path);
/// assert_eq!(filename, "document.txt");
/// ```
pub fn get_file_name(file_path: PathBuf) -> String {
    file_path.file_name().and_then(|os_str| os_str.to_str()).unwrap_or("unknown_file").to_string()
}

/// Generates a random alphanumeric string of the specified length.
///
/// # Arguments
///
/// * `length` - The desired length of the generated string
///
/// # Returns
///
/// A String containing random alphanumeric characters (a-z, A-Z, 0-9)
///
/// # Examples
///
/// ```ignore
/// let random_id = generate_random_string(8);
/// assert_eq!(random_id.len(), 8);
/// ```
pub fn generate_random_string(length: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), length)
}