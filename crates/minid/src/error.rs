//! Error types for the minid runtime.

use std::path::PathBuf;

/// All errors that can occur during minid operations.
#[derive(Debug, thiserror::Error)]
pub enum MinidError {
    /// I/O error (file read/write, directory creation, etc.)
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse or serialize JSON.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error originating from a Linux syscall via the `nix` crate.
    #[error("syscall error: {0}")]
    Nix(#[from] nix::Error),

    /// OCI bundle config.json is invalid or missing required fields.
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// The OCI bundle directory does not exist or is malformed.
    #[error("bundle not found: {}", .0.display())]
    BundleNotFound(PathBuf),

    /// No container exists with the given ID.
    #[error("container not found: {0}")]
    ContainerNotFound(String),

    /// The requested operation is not valid for the container's current state.
    #[error("invalid state transition: cannot {action} container in state {current}")]
    InvalidStateTransition {
        /// The operation that was attempted.
        action: String,
        /// The container's current status.
        current: String,
    },

    /// Error interacting with the cgroup filesystem.
    #[error("cgroup error: {0}")]
    Cgroup(String),
}
