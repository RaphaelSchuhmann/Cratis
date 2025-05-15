use notify::{recommended_watcher, Event, RecursiveMode, Result, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpmc::RecvTimeoutError;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};
// TODO:
// - Use Cratis-Core Error for error handling
// - Load watch directories and exclude directories form config

fn main() {
    let watch_path = "/home/raphael/test/";

    let handle = thread::spawn(move || {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    // Send event to channel
                    tx.send(event).unwrap();
                }
                Err(e) => eprintln!("watch error: {:?}", e),
            }
        }).expect("Failed to create watcher");

        watcher.watch(watch_path, RecursiveMode::Recursive).expect("Failed to watch directory");

        let debounce_duration = Duration::from_millis(500);
        let mut last_event_time = Instant::now();
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
                    eprintln!("channel error: {:?}", e);
                    break;
                }
            }
        }
    });

    // Meanwhile, main thread can do other things...
    println!("File watcher running on separate thread.");

    handle.join().unwrap();
}
