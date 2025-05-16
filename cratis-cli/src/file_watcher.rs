use notify::{recommended_watcher, Event, RecursiveMode, Result, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpmc::RecvTimeoutError;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};
use cratis_core::error::{display_error, CratisError};
// TODO:
// - Load watch directories and exclude directories form config
// - Exclude any directories mentioned in the "exclude" section in the cratis.yml

fn main() {
    let watch_path = "/insert/watch/path/here";

    let handle = thread::spawn(move || {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    // Send event to channel
                    tx.send(event).unwrap();
                }
                Err(e) => display_error(CratisError::WatcherError(!format("{:?}", e)), false),
            }
        }).map_err(|e| display_error(CratisError::WatcherError(!format("Failed to create watcher")), false));

        watcher.watch(watch_path, RecursiveMode::Recursive).map_err(|e| display_error(CratisError::WatchError(!format("Failed to watch directory: {}", watch_path))));

        let debounce_duration: Duration = Duration::from_millis(500);
        let mut last_event_time: Instant = Instant::now();
        let mut pending_events = HashSet::new();

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    for path in event.paths {
                        pending_events.insert(path);
                    }
                    last_event_time = Instant::now();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
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
                    display_error(CratisError::WatcherError(!format("channel error: {:?}", e)), false);
                    break;
                }
            }
        }
    });

    // Meanwhile, main thread can do other things...
    println!("File watcher running on separate thread.");

    handle.join().unwrap();
}
