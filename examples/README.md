# boson-coordinator examples

Canonical teaching path uses in-memory queue/runtime backends; examples that bind HTTP or
start workers say so explicitly.

## 1. Local enqueue — `local_enqueue`

Run when you want to confirm `CoordinatorAdapter` registers a task, enqueues one job, and
reads it back through `BosonCoordinatorBackend`.

```bash
export CARGO_BUILD_JOBS=1
cargo run -p boson-coordinator --example local_enqueue
```

Success: stdout prints `enqueued example_echo as <job-id>`. The job remains queued for a
worker to claim.

See `examples/local_enqueue.rs` for `MemQueueBackend` + `CoordinatorAdapter::enqueue`.

## 2. Axum router — `axum_require_admin`

Run when you want to confirm `boson_router` + `BosonState` construction with fail-closed admin
auth before mounting under `/api/boson`.

```bash
cargo run -p boson-coordinator --example axum_require_admin --features axum
```

Success: stdout prints `Boson axum router constructed with require_admin + lab token.`
Construction smoke check — nest the router under your Axum app to serve.

See `examples/axum_require_admin.rs`, then mount with real admin secrets and
`require_admin_auth(true)`. Prefer this template over `local_enqueue` for production Axum hosts.
