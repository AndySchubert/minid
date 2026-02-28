# minid

A minimal OCI-style container runtime written in Rust.

---

## What is minid?

**minid** implements the core [OCI runtime specification](https://github.com/opencontainers/runtime-spec) lifecycle operations, providing Linux process isolation through namespaces, resource control via cgroups v2, and filesystem isolation using `pivot_root`.

```mermaid
graph TB
    subgraph "microctl (CLI)"
        CLI[clap CLI]
    end

    subgraph "minid (library)"
        CONFIG[config.rs<br/>OCI spec parsing]
        CONTAINER[container.rs<br/>lifecycle orchestration]
        NS[namespace.rs<br/>PID · mount · UTS · IPC]
        CG[cgroup.rs<br/>memory · cpu]
        MNT[mount.rs<br/>pivot_root · /proc · /dev]
        STATE[state.rs<br/>JSON persistence]
    end

    subgraph "Linux Kernel"
        NAMESPACES[Namespaces]
        CGROUPS[cgroups v2]
        VFS[VFS / mount]
    end

    CLI --> CONTAINER
    CONTAINER --> CONFIG
    CONTAINER --> NS
    CONTAINER --> CG
    CONTAINER --> MNT
    CONTAINER --> STATE
    NS --> NAMESPACES
    CG --> CGROUPS
    MNT --> VFS

    style CLI fill:#7c3aed,color:#fff
    style CONTAINER fill:#6d28d9,color:#fff
    style CONFIG fill:#8b5cf6,color:#fff
    style NS fill:#8b5cf6,color:#fff
    style CG fill:#8b5cf6,color:#fff
    style MNT fill:#8b5cf6,color:#fff
    style STATE fill:#8b5cf6,color:#fff
    style NAMESPACES fill:#1e1b4b,color:#fff
    style CGROUPS fill:#1e1b4b,color:#fff
    style VFS fill:#1e1b4b,color:#fff
```

## Features

| Feature | Description |
|---------|-------------|
| **OCI Lifecycle** | `create` → `start` → `kill` → `delete` + `state` query |
| **Namespace Isolation** | PID, mount, UTS, IPC via `unshare(2)` |
| **cgroups v2** | Memory and CPU limits via direct filesystem writes |
| **Rootfs Pivot** | Bind-mount, `/proc` mount, `pivot_root` into the container |
| **State Persistence** | JSON state at `/run/minid/<id>/state.json` |
| **Structured Logging** | `tracing` with `RUST_LOG` env var control |

## Quick Start

```bash
# Build
make build

# Run all checks (fmt + lint + test + build)
make check

# Create and run a container (requires root + Linux)
sudo microctl create demo1 ./my-bundle
sudo microctl start demo1
sudo microctl state demo1
sudo microctl kill demo1 --signal SIGTERM
sudo microctl delete demo1
```

## Requirements

- **Linux** with cgroups v2 enabled
- **Root** privileges (namespaces, cgroups, pivot_root)
- **Rust** 1.75+
