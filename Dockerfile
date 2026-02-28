FROM rust:1.83-bookworm AS builder

WORKDIR /build
COPY . .
RUN cargo build --workspace

# ── Runtime image ─────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy built binary
COPY --from=builder /build/target/debug/microctl /usr/local/bin/microctl

# Prepare an Alpine rootfs bundle for testing
RUN mkdir -p /opt/test-bundle/rootfs \
    && curl -sL https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.1-x86_64.tar.gz \
    | tar xz -C /opt/test-bundle/rootfs

# Write config.json
RUN printf '%s\n' \
    '{' \
    '  "ociVersion": "1.0.2",' \
    '  "root": { "path": "rootfs", "readonly": false },' \
    '  "process": {' \
    '    "args": ["/bin/echo", "hello from minid!"],' \
    '    "cwd": "/",' \
    '    "env": ["PATH=/usr/bin:/bin"],' \
    '    "user": { "uid": 0, "gid": 0 }' \
    '  },' \
    '  "hostname": "minid-test",' \
    '  "linux": {}' \
    '}' > /opt/test-bundle/config.json

# Write test script
RUN printf '%s\n' \
    '#!/bin/bash' \
    'set -euo pipefail' \
    'echo "=== minid lifecycle test ==="' \
    'echo ""' \
    'echo "→ microctl create test1 /opt/test-bundle"' \
    'microctl create test1 /opt/test-bundle' \
    'echo ""' \
    'echo "→ microctl start test1"' \
    'microctl start test1' \
    'echo ""' \
    'sleep 1' \
    'echo "→ microctl state test1"' \
    'microctl state test1' \
    'echo ""' \
    'echo "→ microctl delete test1"' \
    'microctl delete test1' \
    'echo ""' \
    'echo "=== ✓ lifecycle test passed ==="' \
    > /usr/local/bin/test-lifecycle.sh \
    && chmod +x /usr/local/bin/test-lifecycle.sh

CMD ["test-lifecycle.sh"]
