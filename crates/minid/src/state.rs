//! Container state tracking and persistence.
//!
//! State is persisted as JSON under `/run/minid/<container-id>/state.json`.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::MinidError;

/// Base directory for runtime state files.
const STATE_ROOT: &str = "/run/minid";

/// OCI-defined container statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    /// The container environment is being created.
    Creating,
    /// The runtime has finished the create operation and the user process
    /// has not yet been started.
    Created,
    /// The container process is running.
    Running,
    /// The container process has exited.
    Stopped,
}

impl fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creating => write!(f, "creating"),
            Self::Created => write!(f, "created"),
            Self::Running => write!(f, "running"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// Persistent container state (serialised to JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerState {
    /// OCI spec version.
    #[serde(rename = "ociVersion")]
    pub oci_version: String,
    /// Unique container identifier.
    pub id: String,
    /// Current lifecycle status.
    pub status: ContainerStatus,
    /// PID of the container's init process (0 when stopped or not yet started).
    pub pid: u32,
    /// Absolute path to the OCI bundle.
    pub bundle: PathBuf,
    /// Timestamp when the container was created.
    #[serde(rename = "created")]
    pub created_at: DateTime<Utc>,
}

impl ContainerState {
    /// Create a new state in the `Creating` status.
    pub fn new(id: &str, bundle: &Path) -> Self {
        Self {
            oci_version: "1.0.2".to_string(),
            id: id.to_string(),
            status: ContainerStatus::Creating,
            pid: 0,
            bundle: bundle.to_path_buf(),
            created_at: Utc::now(),
        }
    }

    /// Directory where this container's state files are stored.
    pub fn state_dir(id: &str) -> PathBuf {
        PathBuf::from(STATE_ROOT).join(id)
    }

    /// Full path to the state JSON file.
    pub fn state_file(id: &str) -> PathBuf {
        Self::state_dir(id).join("state.json")
    }

    /// Persist this state to disk.
    pub fn save(&self) -> crate::Result<()> {
        let dir = Self::state_dir(&self.id);
        fs::create_dir_all(&dir)?;
        let path = Self::state_file(&self.id);
        let json = serde_json::to_string_pretty(self)?;
        debug!(?path, "saving container state");
        fs::write(&path, json)?;
        Ok(())
    }

    /// Load a container's state from disk.
    pub fn load(id: &str) -> crate::Result<Self> {
        let path = Self::state_file(id);
        if !path.exists() {
            return Err(MinidError::ContainerNotFound(id.to_string()));
        }
        let json = fs::read_to_string(&path)?;
        let state: Self = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// Remove all state files for a container.
    pub fn remove(id: &str) -> crate::Result<()> {
        let dir = Self::state_dir(id);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_display() {
        assert_eq!(ContainerStatus::Creating.to_string(), "creating");
        assert_eq!(ContainerStatus::Created.to_string(), "created");
        assert_eq!(ContainerStatus::Running.to_string(), "running");
        assert_eq!(ContainerStatus::Stopped.to_string(), "stopped");
    }

    #[test]
    fn state_roundtrip_json() {
        let state = ContainerState::new("test-1", Path::new("/tmp/bundle"));
        let json = serde_json::to_string(&state).unwrap();
        let deser: ContainerState = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.id, "test-1");
        assert_eq!(deser.status, ContainerStatus::Creating);
        assert_eq!(deser.pid, 0);
    }

    #[test]
    fn state_dir_paths() {
        assert_eq!(
            ContainerState::state_dir("abc"),
            PathBuf::from("/run/minid/abc")
        );
        assert_eq!(
            ContainerState::state_file("abc"),
            PathBuf::from("/run/minid/abc/state.json")
        );
    }
}
