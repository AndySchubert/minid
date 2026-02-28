# Architecture

## Overview

**minid** is a minimal OCI-style container runtime written in Rust. It isolates
processes using Linux namespaces, controls resource usage via cgroups v2, and
follows the OCI runtime specification lifecycle.

## Crate Layout

```mermaid
graph LR
    subgraph "Cargo Workspace"
        subgraph "crates/microctl"
            MAIN["main.rs<br/>clap CLI"]
        end
        subgraph "crates/minid"
            LIB["lib.rs"]
            ERR["error.rs"]
            CFG["config.rs"]
            ST["state.rs"]
            CTR["container.rs"]
            NS["namespace.rs"]
            CG["cgroup.rs"]
            MT["mount.rs"]
        end
    end

    MAIN --> LIB
    LIB --> CTR
    CTR --> CFG
    CTR --> NS
    CTR --> CG
    CTR --> MT
    CTR --> ST
    CTR --> ERR

    style MAIN fill:#7c3aed,color:#fff
    style LIB fill:#6d28d9,color:#fff
    style CTR fill:#5b21b6,color:#fff
    style CFG fill:#8b5cf6,color:#fff
    style NS fill:#8b5cf6,color:#fff
    style CG fill:#8b5cf6,color:#fff
    style MT fill:#8b5cf6,color:#fff
    style ST fill:#8b5cf6,color:#fff
    style ERR fill:#a78bfa,color:#fff
```

## Data Flow

```mermaid
sequenceDiagram
    participant User
    participant microctl
    participant minid
    participant Kernel

    User->>microctl: microctl create <id> <bundle>
    microctl->>minid: Container::create()
    minid->>minid: config::load_spec()
    minid->>Kernel: fork()
    Kernel-->>minid: child PID
    minid->>Kernel: unshare(PID|NS|UTS|IPC)
    minid->>Kernel: pivot_root()
    minid->>Kernel: cgroup create + add_pid
    minid-->>microctl: ContainerState (Created)

    User->>microctl: microctl start <id>
    microctl->>minid: Container::start()
    minid->>Kernel: signal child → execvpe()
    minid-->>microctl: ok (Running)

    User->>microctl: microctl kill <id>
    microctl->>minid: Container::kill()
    minid->>Kernel: kill(pid, SIGTERM)

    User->>microctl: microctl delete <id>
    microctl->>minid: Container::delete()
    minid->>Kernel: cgroup delete
    minid->>minid: state::remove()
```

## Container Lifecycle State Machine

```mermaid
stateDiagram-v2
    [*] --> Creating : create()
    Creating --> Created : fork + setup complete
    Created --> Running : start()
    Running --> Stopped : process exits / kill
    Stopped --> [*] : delete()
    Created --> Stopped : kill()
```

## State Persistence

Container state is stored as JSON at `/run/minid/<container-id>/state.json`,
following OCI conventions:

| Field | Description |
|-------|-------------|
| `ociVersion` | Spec version (`1.0.2`) |
| `id` | Container ID |
| `status` | `creating` / `created` / `running` / `stopped` |
| `pid` | Init process PID |
| `bundle` | Absolute path to the OCI bundle |
| `created` | ISO 8601 timestamp |
