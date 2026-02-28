.PHONY: build build-release test lint fmt check clean install docs docs-serve docs-deploy test-image test-e2e

# ── Build ─────────────────────────────────────────────
build:
	cargo build --workspace

build-release:
	cargo build --release --workspace

# ── Quality ───────────────────────────────────────────
test:
	cargo test --workspace

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# ── All-in-one CI check ──────────────────────────────
check: fmt-check lint test build

# ── Docker-based E2E test ─────────────────────────────
test-image:
	docker build -t minid-test .

test-e2e: test-image
	docker run --rm --privileged minid-test

test-shell: test-image
	docker run --rm -it --privileged minid-test /bin/bash

# ── Install ───────────────────────────────────────────
install:
	cargo install --path crates/microctl

# ── Docs ──────────────────────────────────────────────
docs:
	mkdocs build --strict

docs-serve:
	mkdocs serve

docs-deploy:
	mkdocs gh-deploy --force

# ── Clean ─────────────────────────────────────────────
clean:
	cargo clean
	rm -rf site/
