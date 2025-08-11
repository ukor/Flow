# Change Log

This document tracks the major architectural decisions and development progress for the Flow project. 

## NOTE: 

This content is not wholly accurate and is only for reference. Will be cleaned up in the future. 


## 2025-07-08: Go Module Consolidation

- **Goal:** Simplify the project's dependency management structure.
- **Change:**
    - Consolidated all dependencies from `core/go.mod` into the root `go.mod` file.
    - Deleted the redundant `core/go.mod` and `core/go.sum` files.
    - Deleted the now-unnecessary `go.work` and `go.work.sum` files, as the project is now a single, unified module.
- **Outcome:** The project now uses a standard single-module structure, which is simpler to manage and understand.

## 2025-07-08: Refactoring Sync Model to be Production-Ready

The initial sync model was a prototype that used an `LWWRegister` to store the entire content of a file as a single blob. This was a form of technical debt, as it was not scalable for large files. A major refactoring effort was undertaken to address this.

### Key Changes:
- **New `Sequence` CRDT (`core/crdt`):**
  - Implemented a robust, ordered-sequence CRDT based on a linked-list model.
  - This allows for efficient, conflict-free management of ordered data like file chunks or directory listings.
  - Covered by comprehensive unit tests (`TestSequence`).

- **Content-Defined Chunking (`core/fscrdt`):**
  - Integrated the `boxo/chunker` library to perform Rabin content-defined chunking.
  - Files are now broken down into variable-sized chunks, identified by their SHA256 hash. This means only modified parts of a file need to be synchronized.

- **Efficient Diffing (`core/fscrdt`):**
  - Replaced a naive diffing library with `go-difflib`.
  - The `FSCRDT` translator now compares the old and new lists of chunk IDs and generates a minimal set of `Insert` and `Remove` operations.
  - This significantly reduces the amount of data that needs to be transmitted for small changes to large files.

- **Full Event Handling:**
  - Implemented and tested handlers for `Create`, `Modify`, and `Delete` filesystem events.

## 2025-07-08: Foundational Implementations

- **Filesystem Watcher (`core/fswatch`):**
  - Built a production-ready, recursive filesystem watcher using the `fsnotify` library.
  - Includes graceful shutdown and comprehensive tests.

- **Identity Management (`core/auth`):**
  - Implemented a robust identity system based on W3C DID standards.
  - On first run, the system generates an `ed25519` keypair and creates a `did:key`.
  - Keys are stored securely in the user's standard configuration directory.
  - The implementation was designed to be testable via monkey-patching for platform-specific code. 