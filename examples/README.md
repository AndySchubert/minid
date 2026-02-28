# Examples

This directory is a placeholder for OCI bundle examples.

## Creating a Minimal OCI Bundle

To test `microctl`, you need an OCI-compliant bundle (a directory with
`config.json` and a `rootfs/`).

### Quick Start with Alpine

```bash
# Create bundle directory
mkdir -p my-bundle/rootfs

# Download Alpine rootfs
curl -sL https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.1-x86_64.tar.gz | \
  tar xz -C my-bundle/rootfs

# Generate a minimal config.json
cat > my-bundle/config.json <<'EOF'
{
  "ociVersion": "1.0.2",
  "root": {
    "path": "rootfs",
    "readonly": true
  },
  "process": {
    "args": ["/bin/sh"],
    "cwd": "/",
    "env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
      "TERM=xterm"
    ]
  },
  "hostname": "minid-demo"
}
EOF

# Run with microctl (requires root)
sudo microctl create demo1 my-bundle
sudo microctl start demo1
sudo microctl state demo1
sudo microctl kill demo1 --signal SIGTERM
sudo microctl delete demo1
```
