//! `minid` — core library for a minimal OCI-style container runtime.
//!
//! This crate implements the fundamental building blocks for running
//! Linux containers following the OCI runtime specification:
//!
//! - **Config**: OCI bundle `config.json` parsing and validation
//! - **State**: Container state machine and persistence
//! - **Container**: Lifecycle orchestration (create / start / kill / delete)
//! - **Namespace**: Linux namespace isolation
//! - **Cgroup**: cgroups v2 resource limits
//! - **Mount**: Rootfs setup and pivot_root

pub mod cgroup;
pub mod config;
pub mod container;
pub mod error;
pub mod mount;
pub mod namespace;
pub mod state;

pub use container::Container;
pub use error::MinidError;
pub use nix::sys::signal::Signal;
pub use state::{ContainerState, ContainerStatus};

/// Convenience result type for this crate.
pub type Result<T> = std::result::Result<T, MinidError>;
