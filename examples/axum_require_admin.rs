//! Construct the Boson Axum bridge with require-admin + static token (lab smoke check).
//!
//! Run with `cargo run --example axum_require_admin --features axum`.

#![allow(clippy::print_stdout)]

use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use boson_backend_mem::MemQueueBackend;
use boson_coordinator::axum_api::{boson_router, BosonState, StaticTokenAdminAuth};
use boson_core::{JsonExecutionContextFactory, QueueBackend, QueueRouter};
use boson_runtime::Boson;

#[tokio::main]
async fn main() -> Result<()> {
    let queue_backend: Arc<dyn QueueBackend> = Arc::new(MemQueueBackend::new());
    QueueRouter::set_global(QueueRouter::with_default(queue_backend));

    let runtime = Arc::new(
        Boson::builder()
            .queue_backend_from_global()
            .execution_context_factory(JsonExecutionContextFactory)
            .auto_registry()
            .build()?,
    );

    let state = BosonState::builder(runtime)
        .admin_auth(Arc::new(StaticTokenAdminAuth::new("lab-only-token")))
        .require_admin_auth(true)
        .build()?;
    let inner = state.inner_axum();
    let _app: Router<boson_axum::BosonState> =
        boson_router::<boson_axum::BosonState>().with_state(inner);

    println!("Boson axum router constructed with require_admin + lab token.");
    Ok(())
}
