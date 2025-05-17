#[allow(unused_must_use)]

use notify::{RecommendedWatcher, Event, RecursiveMode, Result, Watcher};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};
use cratis_core::error::{display_error, CratisError};
use cratis_core::config::{get_config, load_config, CratisConfig}; // Remove load_config() once config loading is properly implemented
use cratis_core::utils::{EventAction, map_event_kinds};
use glob::Pattern;

fn main() {
    let _ = load_config("/home/raphael/Development/Cratis/cratis.yml");

    let config: &CratisConfig = get_config();

    let (tx, rx) = channel();

    let watch_dirs: &Vec<String> = &config.backup.watch_directories;
    let exclude_dirs: &Vec<String> = &config.backup.exclude.clone().unwrap();

    let mut exclude_patterns: Vec<Pattern> = Vec::new();
    
    if !exclude_dirs.is_empty() {
        exclude_patterns.extend(exclude_dirs.iter().map(|pattern| Pattern::new(pattern).unwrap()));
    }
    
    let _watcher = start_watching(&watch_dirs, tx).unwrap();

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
                            pending_events.insert((path, event_action));
                        },
                        _ => {
                            if let Ok(metadata) = fs::metadata(&path) {
                                if metadata.is_file() {
                                    pending_events.insert((path, event_action));
                                }
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

fn is_excluded(path: &Path, exclude_patterns: &[Pattern]) -> bool {
    exclude_patterns.iter().any(|pattern| pattern.matches_path(path))
}