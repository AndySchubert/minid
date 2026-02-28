# minid

A minimal OCI-style container runtime written in Rust.

## Overview

**minid** implements the core [OCI runtime specification](https://github.com/opencontainers/runtime-spec) lifecycle operations, providing Linux process isolation through namespaces, resource control via cgroups v2, and filesystem isolation using `pivot_root`.

```
┌─────────────────────────────────────────────────┐
│  microctl (CLI)                                 │
│  ┌───────────────────────────────────────────┐  │
│  │  minid (library)                          │  │
│  │  ┌─────────┐ ┌──────────┐ ┌───────────┐   │  │
│  │  │Namespace│ │ Cgroup   │ │ Mount     │   │  │
│  │  │  PID    │ │ memory   │ │ pivot_root│   │  │
│  │  │  mount  │ │ cpu      │ │ /proc     │   │  │
│  │  │  UTS    │ │          │ │ /dev      │   │  │
│  │  │  IPC    │ │          │ │           │   │  │
│  │  └─────────┘ └──────────┘ └───────────┘   │  │
│  └───────────────────────────────────────────┘  │
│                    Linux Kernel                 │
└─────────────────────────────────────────────────┘
```

## Features

- **OCI Lifecycle** — `create` → `start` → `kill` → `delete` + `state` query
- **Namespace Isolation** — PID, mount, UTS, IPC via `unshare(2)`
- **cgroups v2** — Memory and CPU limits via direct filesystem writes
- **Rootfs Pivot** — Bind-mount, `/proc` mount, `pivot_root` into the container
- **State Persistence** — JSON state at `/run/minid/<id>/state.json`
- **Structured Logging** — `tracing` with `RUST_LOG` env var control

## Project Structure

```
crates/
  minid/               # library: core runtime engine
    src/
      lib.rs            # public API
      error.rs          # MinidError (thiserror)
      config.rs         # OCI config.json loading
      state.rs          # container state machine
      container.rs      # lifecycle orchestration
      namespace.rs      # namespace setup
      cgroup.rs         # cgroups v2
      mount.rs          # rootfs / pivot_root
  microctl/             # binary: CLI frontend
    src/main.rs          # clap CLI
docs/
  architecture.md       # system architecture
  security.md           # security model
  design-decisions.md   # rationale & trade-offs
examples/               # OCI bundle examples
tests/                  # integration tests (require root)
```

## Quick Start

### Build

```bash
cargo build --workspace
```

### Usage (requires root)

```bash
# Create an OCI bundle (see examples/README.md)
mkdir -p my-bundle/rootfs
# ... populate rootfs ...

# Lifecycle
sudo ./target/debug/microctl create demo1 ./my-bundle
sudo ./target/debug/microctl start demo1
sudo ./target/debug/microctl state demo1
sudo ./target/debug/microctl kill demo1 --signal SIGTERM
sudo ./target/debug/microctl delete demo1
```

### CLI Reference

```
microctl create <id> <bundle>     Create a container from an OCI bundle
microctl start <id>               Start a created container
microctl state <id>               Query container state (JSON)
microctl kill <id> [-s SIGNAL]    Send signal to container (default: SIGTERM)
microctl delete <id>              Delete a stopped container
```

## Development

```bash
cargo fmt --all             # format code
cargo clippy --workspace    # lint
cargo test --workspace      # run tests
```

## Requirements

- **Linux** with cgroups v2 enabled
- **Root** privileges (namespaces, cgroups, pivot_root)
- **Rust** 1.75+

## Documentation

- [Architecture](docs/architecture.md) — crate layout, data flow, state machine
- [Security](docs/security.md) — isolation model and known limitations
- [Design Decisions](docs/design-decisions.md) — rationale behind key choices

## License

MIT
