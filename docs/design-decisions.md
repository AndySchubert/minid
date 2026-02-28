# Design Decisions

## Why Rust?

```mermaid
mindmap
  root((Rust))
    Memory Safety
      No GC pauses
      Ownership model
      No use-after-free
    Performance
      Zero-cost abstractions
      C-comparable speed
      Small binary size
    Ecosystem
      nix crate for syscalls
      oci-spec for config
      serde for serialization
    Reliability
      Enum-based errors
      Pattern matching
      Compiler-enforced safety
```

- **Memory safety without GC** — container runtimes are long-lived,
  performance-critical processes. Rust's ownership model prevents the class
  of memory bugs (use-after-free, double-free, buffer overflows) that plague
  C-based runtimes.
- **Zero-cost abstractions** — enum-based error handling, trait-based
  polymorphism, and iterators compile down to the same code you'd write in C.
- **Ecosystem** — the `nix` crate provides idiomatic, safe wrappers around
  Linux syscalls. The `oci-spec` crate handles spec parsing.

## Why a Cargo Workspace?

```mermaid
graph LR
    subgraph "Consumers"
        CLI["microctl<br/>(binary)"]
        LIB_USER["Future tools<br/>(library users)"]
    end
    subgraph "Core"
        MINID["minid<br/>(library)"]
    end

    CLI --> MINID
    LIB_USER --> MINID

    style CLI fill:#a78bfa,color:#fff
    style LIB_USER fill:#c4b5fd,color:#1e1b4b
    style MINID fill:#8b5cf6,color:#fff
```

Splitting into `minid` (library) and `microctl` (binary) provides:

1. **Testability** — the core library can be unit-tested independently of
   the CLI.
2. **Reusability** — other tools can depend on `minid` as a library crate.
3. **Separation of concerns** — CLI argument parsing, output formatting, and
   error presentation live in `microctl`, while container logic lives in
   `minid`.

## Why cgroups v2 Only?

!!! info "cgroups v2 is the modern standard"
    All modern Linux distributions (Ubuntu 22.04+, Fedora 31+, Debian 11+)
    default to cgroups v2.

- cgroups v1 is a legacy interface with a fragmented hierarchy (one mount
  per controller). v2 uses a unified hierarchy that's simpler to manage.
- Keeping v1 support would double the cgroup code for diminishing returns.

## Why No seccomp in v1?

seccomp-bpf requires either:

- A BPF program compiled from a policy file, or
- Integration with `libseccomp` (C library with Rust bindings)

Both add significant complexity. The goal of v1 is to demonstrate core
container mechanics (namespaces, cgroups, rootfs). seccomp is the natural
next step for a v2.

## Fork vs Clone

We use `fork()` + `unshare()` rather than `clone(CLONE_NEW*)` because:

1. `nix::unistd::fork()` is a safe wrapper available on stable Rust.
2. `clone()` with namespace flags requires `unsafe` and careful stack
   management.
3. The two-step approach (fork then unshare) is easier to debug and matches
   what runc does internally.

## Socket-Based Start Synchronisation

```mermaid
sequenceDiagram
    participant Parent as Parent (microctl)
    participant Socket as Unix Socket Pair
    participant Child as Child (container)

    Parent->>Socket: create pair
    Parent->>Child: fork()
    Child->>Child: unshare() + rootfs setup
    Child->>Socket: read() — blocks

    Note over Parent: Container is "Created"

    Parent->>Socket: write(1) — start signal
    Socket-->>Child: unblocked
    Child->>Child: execvpe(user process)

    Note over Child: Container is "Running"
```

The `create` → `start` two-phase lifecycle requires the child process to
wait after setup before exec'ing the user workload. We use a Unix socket
pair:

- **create**: parent and child each get one end. The child blocks on `read()`.
- **start**: the parent writes a byte, unblocking the child to call `exec`.

This avoids PID file race conditions and is more reliable than signal-based
synchronisation.

## State Persistence at /run/minid/

We store state under `/run/minid/<id>/state.json` because:

- `/run` is a tmpfs that's cleaned on reboot (no stale state).
- It matches the convention used by runc (`/run/runc`).
- JSON is human-readable and easy to debug.
