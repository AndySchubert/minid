//! cgroups v2 resource management.
//!
//! Manages cgroup directories under `/sys/fs/cgroup/minid/<container-id>/`.
//! Supports memory and CPU limits via direct filesystem writes.

use std::fs;
use std::path::PathBuf;

use tracing::debug;

use crate::error::MinidError;

/// Root path for the minid cgroup hierarchy (cgroups v2).
const CGROUP_ROOT: &str = "/sys/fs/cgroup/minid";

/// Resource limits that can be applied to a container's cgroup.
#[derive(Debug, Clone, Default)]
pub struct Resources {
    /// Memory limit in bytes. `None` means unlimited.
    pub memory_limit: Option<u64>,
    /// CPU weight (1–10000). Maps to `cpu.weight`. `None` means default.
    pub cpu_weight: Option<u64>,
}

/// Manages cgroup lifecycle for a single container.
#[derive(Debug)]
pub struct CgroupManager {
    /// Container ID this manager is responsible for.
    id: String,
}

impl CgroupManager {
    /// Create a new cgroup manager for the given container ID.
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }

    /// Cgroup directory for this container.
    fn cgroup_path(&self) -> PathBuf {
        PathBuf::from(CGROUP_ROOT).join(&self.id)
    }

    /// Create the cgroup directory.
    pub fn create(&self) -> crate::Result<()> {
        let path = self.cgroup_path();
        debug!(?path, "creating cgroup directory");
        fs::create_dir_all(&path).map_err(|e| {
            MinidError::Cgroup(format!("failed to create cgroup dir: {e}"))
        })?;
        Ok(())
    }

    /// Apply resource limits by writing to cgroup control files.
    pub fn apply(&self, resources: &Resources) -> crate::Result<()> {
        let path = self.cgroup_path();

        if let Some(mem) = resources.memory_limit {
            let mem_path = path.join("memory.max");
            debug!(?mem_path, bytes = mem, "setting memory limit");
            fs::write(&mem_path, mem.to_string()).map_err(|e| {
                MinidError::Cgroup(format!("failed to set memory.max: {e}"))
            })?;
        }

        if let Some(weight) = resources.cpu_weight {
            let cpu_path = path.join("cpu.weight");
            debug!(?cpu_path, weight, "setting cpu weight");
            fs::write(&cpu_path, weight.to_string()).map_err(|e| {
                MinidError::Cgroup(format!("failed to set cpu.weight: {e}"))
            })?;
        }

        Ok(())
    }

    /// Add a process to this cgroup.
    pub fn add_pid(&self, pid: u32) -> crate::Result<()> {
        let procs_path = self.cgroup_path().join("cgroup.procs");
        debug!(?procs_path, pid, "adding PID to cgroup");
        fs::write(&procs_path, pid.to_string()).map_err(|e| {
            MinidError::Cgroup(format!("failed to add pid to cgroup: {e}"))
        })?;
        Ok(())
    }

    /// Delete the cgroup directory (container must have no running processes).
    pub fn delete(&self) -> crate::Result<()> {
        let path = self.cgroup_path();
        if path.exists() {
            debug!(?path, "removing cgroup directory");
            fs::remove_dir(&path).map_err(|e| {
                MinidError::Cgroup(format!("failed to remove cgroup dir: {e}"))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgroup_path_construction() {
        let mgr = CgroupManager::new("test-container");
        assert_eq!(
            mgr.cgroup_path(),
            PathBuf::from("/sys/fs/cgroup/minid/test-container")
        );
    }

    #[test]
    fn default_resources() {
        let r = Resources::default();
        assert!(r.memory_limit.is_none());
        assert!(r.cpu_weight.is_none());
    }
}
