//! Container lifecycle orchestration.
//!
//! Implements the OCI runtime operations: create, start, kill, delete, state.

use std::io::Read;
use std::os::unix::net::UnixStream;
use std::path::Path;

use nix::sys::signal::{self, Signal};
use nix::sys::wait::{self, WaitPidFlag, WaitStatus};
use nix::unistd::{self, ForkResult, Pid};
use tracing::{debug, info};

use crate::cgroup::CgroupManager;
use crate::config;
use crate::error::MinidError;
use crate::mount;
use crate::namespace;
use crate::state::{ContainerState, ContainerStatus};

/// High-level container lifecycle manager.
pub struct Container;

impl Container {
    /// Create a new container from an OCI bundle.
    ///
    /// This forks a child process into new namespaces, sets up the rootfs
    /// and cgroup, then waits for a `start` signal before exec'ing the
    /// user workload. The container enters the `Created` state.
    pub fn create(id: &str, bundle_path: &Path) -> crate::Result<ContainerState> {
        info!(id, ?bundle_path, "creating container");

        // Load and validate the OCI spec.
        let spec = config::load_spec(bundle_path)?;
        let root = spec
            .root()
            .as_ref()
            .expect("validated in config::load_spec");
        let rootfs = bundle_path.join(root.path());

        let process = spec
            .process()
            .as_ref()
            .expect("validated in config::load_spec");
        let args: Vec<String> = process
            .args()
            .as_ref()
            .expect("validated")
            .clone();

        // Create a Unix socket pair for parent↔child synchronisation.
        // The child will block reading from its end until `start` is called.
        let (parent_sock, child_sock) = UnixStream::pair()?;

        // Set up cgroup.
        let cgroup = CgroupManager::new(id);
        cgroup.create()?;

        // Fork into a child process.
        match unsafe { unistd::fork() }? {
            ForkResult::Parent { child } => {
                drop(child_sock);
                let pid = child.as_raw() as u32;
                info!(id, pid, "child process forked");

                // Add child to cgroup.
                cgroup.add_pid(pid)?;

                // Persist state as Created.
                let mut state = ContainerState::new(id, bundle_path);
                state.status = ContainerStatus::Created;
                state.pid = pid;
                state.save()?;

                // Keep the parent socket alive by storing its fd path in the
                // state directory so `start` can write to it.
                let sock_path = ContainerState::state_dir(id).join("start.sock");
                // We serialize the parent socket fd — in a real runtime we'd
                // use a named socket. For simplicity we save the socket.
                Self::persist_socket(parent_sock, &sock_path)?;

                Ok(state)
            }
            ForkResult::Child => {
                drop(parent_sock);

                // Child: set up namespaces.
                if let Err(e) = namespace::setup_namespaces(namespace::DEFAULT_CLONE_FLAGS) {
                    eprintln!("minid: namespace setup failed: {e}");
                    std::process::exit(1);
                }

                // Set hostname.
                let hostname = spec
                    .hostname()
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or(id);
                if let Err(e) = namespace::set_hostname(hostname) {
                    eprintln!("minid: set_hostname failed: {e}");
                    std::process::exit(1);
                }

                // Set up rootfs.
                if let Err(e) = mount::setup_rootfs(&rootfs) {
                    eprintln!("minid: rootfs setup failed: {e}");
                    std::process::exit(1);
                }

                // Block until parent signals us to start via the socket.
                let mut buf = [0u8; 1];
                let _ = (&child_sock).read(&mut buf);

                // Exec the user process.
                let cargs: Vec<std::ffi::CString> = args
                    .iter()
                    .map(|a| std::ffi::CString::new(a.as_str()).unwrap())
                    .collect();

                let env: Vec<std::ffi::CString> = process
                    .env()
                    .as_ref()
                    .map(|e| {
                        e.iter()
                            .map(|v| std::ffi::CString::new(v.as_str()).unwrap())
                            .collect()
                    })
                    .unwrap_or_default();

                // This should not return.
                let _ = unistd::execvpe(&cargs[0], &cargs, &env);
                eprintln!("minid: execvpe failed");
                std::process::exit(1);
            }
        }
    }

    /// Start a previously created container.
    ///
    /// Signals the child process to exec the user workload.
    pub fn start(id: &str) -> crate::Result<()> {
        info!(id, "starting container");

        let mut state = ContainerState::load(id)?;
        if state.status != ContainerStatus::Created {
            return Err(MinidError::InvalidStateTransition {
                action: "start".into(),
                current: state.status.to_string(),
            });
        }

        // Signal the child by writing to the start socket.
        let sock_path = ContainerState::state_dir(id).join("start.sock");
        Self::signal_start(&sock_path)?;

        state.status = ContainerStatus::Running;
        state.save()?;

        Ok(())
    }

    /// Query the current state of a container.
    pub fn state(id: &str) -> crate::Result<ContainerState> {
        let mut state = ContainerState::load(id)?;

        // If the container is running, check if the process is still alive.
        if state.status == ContainerStatus::Running && state.pid > 0 {
            let pid = Pid::from_raw(state.pid as i32);
            match wait::waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(..)) | Ok(WaitStatus::Signaled(..)) => {
                    debug!(id, "process exited, updating state to stopped");
                    state.status = ContainerStatus::Stopped;
                    state.pid = 0;
                    state.save()?;
                }
                _ => {}
            }
        }

        Ok(state)
    }

    /// Send a signal to the container's init process.
    pub fn kill(id: &str, sig: Signal) -> crate::Result<()> {
        info!(id, ?sig, "killing container");

        let state = ContainerState::load(id)?;
        if state.status != ContainerStatus::Running && state.status != ContainerStatus::Created {
            return Err(MinidError::InvalidStateTransition {
                action: "kill".into(),
                current: state.status.to_string(),
            });
        }

        if state.pid > 0 {
            let pid = Pid::from_raw(state.pid as i32);
            signal::kill(pid, sig)?;
        }

        Ok(())
    }

    /// Delete a stopped container, cleaning up all resources.
    pub fn delete(id: &str) -> crate::Result<()> {
        info!(id, "deleting container");

        let state = ContainerState::load(id)?;
        if state.status != ContainerStatus::Stopped {
            return Err(MinidError::InvalidStateTransition {
                action: "delete".into(),
                current: state.status.to_string(),
            });
        }

        // Clean up cgroup.
        let cgroup = CgroupManager::new(id);
        cgroup.delete()?;

        // Remove state files.
        ContainerState::remove(id)?;

        Ok(())
    }

    /// Persist the parent-end socket so `start` can retrieve it.
    fn persist_socket(sock: UnixStream, path: &Path) -> crate::Result<()> {
        use std::os::unix::io::AsRawFd;
        // Store the fd number so we can recreate it later.
        let fd = sock.as_raw_fd();
        std::fs::write(path, fd.to_string())?;
        // Leak the socket so the fd stays open.
        std::mem::forget(sock);
        Ok(())
    }

    /// Signal the child to start by writing to the persisted socket fd.
    fn signal_start(path: &Path) -> crate::Result<()> {
        use std::io::Write;
        use std::os::unix::io::FromRawFd;

        let fd_str = std::fs::read_to_string(path)?;
        let fd: i32 = fd_str.trim().parse().map_err(|e| {
            MinidError::InvalidConfig(format!("invalid socket fd: {e}"))
        })?;

        let mut sock = unsafe { UnixStream::from_raw_fd(fd) };
        sock.write_all(&[1])?;
        drop(sock);

        // Clean up the fd file.
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
