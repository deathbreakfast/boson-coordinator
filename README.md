# boson-coordinator

[![CI](https://github.com/unified-field-dev/boson-coordinator/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/boson-coordinator/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[GitHub](https://github.com/unified-field-dev/boson-coordinator) · `cargo doc -p boson-coordinator --open`

Object-safe Boson coordinator API: backend trait, local runtime handle, Axum and remote HTTP backends, task-config bootstrap, and autoscale helpers.

```toml
boson-coordinator = { git = "https://github.com/unified-field-dev/boson-coordinator" }
# Optional: features = ["axum"] or ["remote-http"]
```

```rust
use boson_coordinator::{BosonCoordinatorBackend, CoordinatorAdapter};

let backend: Box<dyn BosonCoordinatorBackend> = Box::new(CoordinatorAdapter::new(/* … */));
// Hosts call upsert/list/stats through the trait without depending on concrete backends.
```

## About

- `BosonCoordinatorBackend` — object-safe admin API for task config and coordination
- `CoordinatorAdapter` — local in-process runtime handle
- Axum state and router (`axum`) and HTTP client for split topologies (`remote-http`)
- Task-config bootstrap and autoscale helpers

## Features

| Feature | Purpose |
|---------|---------|
| (default) | Trait, local runtime handle, stats, scaling, task-config bootstrap |
| `axum` | Axum state bridge to `boson-axum` — use `BosonState::builder` + `require_admin_auth(true)` |
| `remote-http` | HTTP client for split topology; signs with `SUBSYSTEM_AUTH_HMAC_KEY` when set |

## Examples

Canonical teaching path and run commands: [examples/README.md](examples/README.md).

## Verify

```bash
export CARGO_BUILD_JOBS=1
# Serialize until shared OnceLock registry/queue fixtures are isolated (docs/VERIFICATION.md).
cargo test -- --test-threads=1
cargo test --features axum -- --test-threads=1
```

## License

MIT. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
