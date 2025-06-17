#![allow(unused_must_use)]
#![allow(unused_imports)]

use notify::{RecommendedWatcher, Event, RecursiveMode, Result, Watcher};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};
use cratis_core::error::{display_error, CratisError};
use cratis_core::config::{get_config, load_config, CratisConfig, TEMP_CONFIG_PATH}; // Remove load_config() once config loading is properly implemented
use cratis_core::utils::{EventAction, map_event_kinds};
use glob::Pattern;

/// Entry point for the Cratis file watcher application.
///
/// This function initializes and runs the file watching system with the following steps:
/// 1. Loads configuration from a specified YAML file
/// 2. Sets up file system watching for configured directories
/// 3. Implements event debouncing with a 500ms window
/// 4. Processes file system events while filtering out temporary files and excluded paths
///
/// # Configuration
///
/// The application expects a configuration file that specifies:
/// * Watch directories to monitor
/// * Directories to exclude from monitoring
///
/// # Error Handling
///
/// The function handles various error cases including:
/// * Configuration loading failures
/// * File system watcher setup errors
/// * Event receiving timeouts
///
/// # Implementation Details
///
/// Uses a channel-based approach for event handling with:
/// * Debouncing mechanism to prevent event flooding
/// * Event filtering for temporary files
/// * Pattern-based exclusion system
fn main() {
    let _ = load_config(TEMP_CONFIG_PATH);

    let mut config = get_config();
    
    let (tx, rx) = channel();

    let watch_dirs: &Vec<String> = &config.backup.watch_directories;
    let exclude_dirs: &Vec<String> = &config.backup.exclude.clone().unwrap_or_default();

    let mut exclude_patterns: Vec<Pattern> = Vec::new();
    
    if !exclude_dirs.is_empty() {
        for pattern in exclude_dirs.iter() {
            match Pattern::new(pattern) {
                Ok(p) => exclude_patterns.push(p),
                Err(e) => display_error(&CratisError::ConfigError(format!("Invalid exclusion pattern '{}': {}", pattern, e)), false)
            }
        }
    }
    
    let _watcher = start_watching(watch_dirs, tx).unwrap();

    let debounce_duration: Duration = Duration::from_millis(500);
    let mut last_event_time: Instant = Instant::now();
    let mut pending_events: HashSet<(std::path::PathBuf, EventAction)> = HashSet::new();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                for path in event.paths {
                    if is_temp_file(&path) || is_excluded(&path, &exclude_patterns) { continue; }

                    let event_action = map_event_kinds(&event.kind);
                    
                    match event_action {
                        EventAction::Delete => {
                            pending_events.insert((path.clone(), event_action));
                        },
                        _ => {
                            if path.exists() {
                                if let Ok(metadata) = fs::metadata(&path) {
                                    if metadata.is_file() {
                                        pending_events.insert((path, event_action));
                                    }
                                }
                            } else {
                                pending_events.insert((path.clone(), EventAction::Delete));
                            }
                        }
                    }
                }
                last_event_time = Instant::now();
            }
            Err(RecvTimeoutError::Timeout) => {
                if !pending_events.is_empty() && last_event_time.elapsed() >= debounce_duration {
                    println!("Batch of changed paths:");
                    for p in &pending_events {
                        println!(" - {:?}", p);
                    }
                    // TODO: Call sync function from here

                    pending_events.clear();
                }
            }
            Err(e) => {
                display_error(&CratisError::ChannelError(format!("{}", e)), false);
                break;
            }
        }
    }
}

/// Initializes and starts a file system watcher for the specified paths.
///
/// Sets up a `RecommendedWatcher` instance that monitors the given paths for file system events
/// and forwards them through a channel.
///
/// # Arguments
///
/// * `paths` - A vector of strings representing the file system paths to watch
/// * `tx` - A channel sender for forwarding file system events
///
/// # Returns
///
/// Returns a `Result` containing the initialized `RecommendedWatcher` if successful.
///
/// # Errors
///
/// This function can fail if:
/// * The watcher cannot be initialized
/// * Any of the specified paths cannot be watched
///
/// # Example
///
/// ```rust
/// use std::sync::mpsc::channel;
///
/// let (tx, rx) = channel();
/// let paths = vec![String::from("/path/to/watch")];
/// let watcher = start_watching(&paths, tx)?;
/// ```
///
/// # Implementation Details
///
/// * Uses recursive watching mode for all directories
/// * Automatically handles error cases by displaying them through `CratisError`
/// * Events are sent through the channel asynchronously
/// * Failed watch attempts for individual paths are logged but don't stop the overall watching process
fn start_watching(paths: &Vec<String>, tx: Sender<Event>) -> Result<RecommendedWatcher> {
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event>| {
            match res {
                Ok(event) => tx.send(event).unwrap(),
                Err(e) => display_error(&CratisError::WatcherError(format!("{:?}", e)), false),
            }
        },
        notify::Config::default(),
    )?;

    for path in paths {
        let _ = watcher.watch(Path::new(path), RecursiveMode::Recursive).map_err(|_| display_error(&CratisError::WatcherError(format!("Failed to watch directory: {}", path)), false));
    }

    Ok(watcher)
}

/// Determines if a given path represents a temporary file.
///
/// Checks the file name against common temporary file patterns used by various
/// text editors and systems.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` to check
///
/// # Returns
///
/// Returns `true` if the file matches any of these patterns:
/// * Files starting with '.'
/// * Files ending with '.tmp', '.temp', '.swp', or '.bak'
/// * Files starting with '~' or '.#'
/// * Vim temporary files (specifically '4913')
///
/// Returns `false` if:
/// * The path has no filename
/// * The filename cannot be converted to a string
/// * The filename doesn't match any temporary file patterns
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
///
/// let temp_file = Path::new(".temporary.swp");
/// assert!(is_temp_file(&temp_file));
///
/// let normal_file = Path::new("document.txt");
/// assert!(!is_temp_file(&normal_file));
/// ```
fn is_temp_file(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        name.starts_with('.')
            || name.ends_with(".tmp")
            || name.ends_with(".temp")
            || name.ends_with(".swp")
            || name.ends_with(".bak")
            || name.starts_with("~")
            || name.starts_with(".#")
            || name == "4913" // vim temp file
    } else {
        false
    }
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
/// ```rust
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
fn is_excluded(path: &Path, exclude_patterns: &[Pattern]) -> bool {
    exclude_patterns.iter().any(|pattern| pattern.matches_path(path))
}