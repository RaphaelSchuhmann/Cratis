# Cratis

**Cratis** is a versioned backup system built in Rust for privacy-focused, local-first environments. It tracks changes in files, stores timestamped versions, and enables powerful time-based snapshot restoration.

---

## Features (WIP)

- Real-time file watching using a cross-platform watcher.
- Timestamp-based versioning of individual files.
- Flat file storage with a lightweight embedded database (powered by `sled`).
- HTTP API to manage uploads, snapshots, and restore operations.
- Local/private use, no cloud dependency.

---
## Authors
[@RaphaelSchuhmann](https://www.github.com/RaphaelSchuhmann)
