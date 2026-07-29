# Contributing to boson-coordinator

Thank you for improving this project.

## Development setup

1. Clone [unified-field-dev/boson-coordinator](https://github.com/unified-field-dev/boson-coordinator)
2. Install Rust stable
3. From the repository root:

```bash
cargo fmt --all -- --check
cargo check
# Prefer --test-threads=1 until integration tests stop sharing process-wide
# OnceLock registries / queue fixtures (see docs/VERIFICATION.md).
cargo test -- --test-threads=1
```

## Code of conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security reports: [`SECURITY.md`](SECURITY.md).

## Pull requests

- Prefer small, focused PRs.
- Update rustdoc and [`README.md`](README.md) when public API or host wiring steps change.
