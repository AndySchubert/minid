//! Linux namespace setup for container isolation.

use nix::sched::{self, CloneFlags};
use nix::unistd;
use tracing::debug;

/// Default namespace flags for container isolation.
///
/// - `CLONE_NEWPID`  — isolated PID namespace (container sees itself as PID 1)
/// - `CLONE_NEWNS`   — isolated mount namespace
/// - `CLONE_NEWUTS`  — isolated hostname / domainname
/// - `CLONE_NEWIPC`  — isolated IPC (semaphores, message queues, shared memory)
pub const DEFAULT_CLONE_FLAGS: CloneFlags = CloneFlags::from_bits_truncate(
    CloneFlags::CLONE_NEWPID.bits()
        | CloneFlags::CLONE_NEWNS.bits()
        | CloneFlags::CLONE_NEWUTS.bits()
        | CloneFlags::CLONE_NEWIPC.bits(),
);

/// Apply namespace isolation to the current process via `unshare(2)`.
///
/// Must be called from the child process before exec'ing the user workload.
pub fn setup_namespaces(flags: CloneFlags) -> crate::Result<()> {
    debug!(?flags, "unsharing namespaces");
    sched::unshare(flags)?;
    Ok(())
}

/// Set the hostname inside the UTS namespace.
pub fn set_hostname(name: &str) -> crate::Result<()> {
    debug!(hostname = name, "setting container hostname");
    unistd::sethostname(name)?;
    Ok(())
}
