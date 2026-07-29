//! Boot an in-memory Boson coordinator and enqueue one task.

#![allow(clippy::print_stdout, clippy::expect_used)]

use std::sync::Arc;

use anyhow::Result;
use boson_backend_mem::MemQueueBackend;
use boson_coordinator::{BosonCoordinatorBackend, CoordinatorAdapter};
use boson_core::{ExecutionContext, JsonExecutionContextFactory, QueueBackend, QueueRouter};
use boson_runtime::Boson;
use serde_json::json;

#[boson_macros::task(name = "example_echo")]
async fn example_echo(_context: Box<dyn ExecutionContext>) -> boson_core::Result<()> {
    Ok(())
}

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
    let coordinator: Arc<dyn BosonCoordinatorBackend> = Arc::new(CoordinatorAdapter::new(runtime));

    let job_id = coordinator
        .enqueue(
            "example_echo",
            json!({"System": {"operation": "getting-started"}}),
            json!({}),
            None,
        )
        .await?;
    let job = coordinator
        .get_job(&job_id)
        .await
        .expect("the newly enqueued job must be present");

    println!("enqueued {} as {}", job.task_name, job.job_id);
    Ok(())
}
