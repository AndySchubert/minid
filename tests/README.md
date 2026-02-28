# Integration Tests

Integration tests for minid require **root** privileges and a **Linux** host
with cgroups v2 enabled. They are intentionally kept separate from unit tests.

## Running

```bash
sudo cargo test --test '*' -- --test-threads=1
```

## Future Tests

- `create` → `state` → `start` → `state` → `kill` → `delete` lifecycle
- Invalid bundle handling
- cgroup limit enforcement
- Concurrent container creation
