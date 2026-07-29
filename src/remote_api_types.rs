//! Remote HTTP API DTOs (mirror `boson_axum` handler shapes).
//!
//! These types are the wire format for [`crate::remote_http::HttpRemoteBosonCoordinatorBackend`]:
//! requests/responses are JSON bodies under `/api/boson/*`, wrapped in an envelope
//! (`{"success": bool, "data": ..., "error": ...}`) on the server side. Timestamps are RFC 3339
//! strings rather than typed `DateTime<Utc>` so this crate does not need to agree on serde
//! `DateTime` representation with every server version; [`crate::remote_http`] parses them back
//! into [`boson_core::Job`] / [`boson_core::Run`] / [`boson_core::TaskConfig`] locally.

use boson_core::{RateLimitPolicy, RetryPolicy};
use serde::{Deserialize, Serialize};

/// Request body for `POST /api/boson/jobs/enqueue`.
#[derive(Debug, Deserialize, Serialize)]
pub struct EnqueueRequest {
    /// Task name; must be registered via `#[boson::task]` inventory on the server.
    pub task_name: String,
    /// Task parameters, serialized to JSON (defaults to `{}` if omitted).
    #[serde(default)]
    pub params: serde_json::Value,
    /// Idempotency key to dedupe repeated enqueue attempts.
    pub idempotency_key: Option<String>,
}

/// Response body for `POST /api/boson/jobs/enqueue`.
#[derive(Debug, Serialize, Deserialize)]
pub struct EnqueueResponse {
    /// Id of the newly created job.
    pub job_id: String,
}

/// Response shape for job read endpoints (`GET /api/boson/jobs`, `GET /api/boson/jobs/{id}`).
#[derive(Debug, Serialize, Deserialize)]
pub struct JobResponse {
    /// Unique job identifier (UUID).
    pub job_id: String,
    /// Task name the job was enqueued for.
    pub task_name: String,
    /// Job status as a lowercase string (`queued`, `running`, `success`, `failed`, `canceled`).
    pub status: String,
    /// Priority (lower value = higher priority).
    pub priority: i32,
    /// Worker pool the job is assigned to.
    pub pool: String,
    /// Creation timestamp, RFC 3339.
    pub created_at: String,
}

/// Response shape for run read endpoints (`GET /api/boson/runs`, `GET /api/boson/runs/{id}`).
#[derive(Debug, Serialize, Deserialize)]
pub struct RunResponse {
    /// Unique run identifier (UUID).
    pub run_id: String,
    /// Job this run belongs to.
    pub job_id: String,
    /// Task name (denormalized from the job).
    pub task_name: String,
    /// Attempt number for this run (1-based).
    pub attempt: i32,
    /// Run status as a lowercase string (`running`, `success`, `failed`, `canceled`, `timeout`).
    pub status: String,
    /// When execution started, RFC 3339.
    pub started_at: String,
    /// When execution finished, RFC 3339 (`None` while still running).
    pub finished_at: Option<String>,
    /// Duration in milliseconds, once finished.
    pub duration_ms: Option<i64>,
}

/// Response shape for `GET /api/boson/tasks/{name}/config`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskConfigResponse {
    /// Task name (unique key).
    pub task_name: String,
    /// Effective priority (lower value = higher priority).
    pub priority: i32,
    /// Effective worker pool name.
    pub pool: String,
    /// Effective retry policy.
    pub retry_policy: RetryPolicy,
    /// Effective rate-limit policy.
    pub rate_limit_policy: RateLimitPolicy,
    /// Last update timestamp, RFC 3339.
    pub updated_at: String,
}

/// Request body for `POST /api/boson/tasks/{name}/config` (partial update; `None` fields are
/// left unchanged server-side).
#[derive(Debug, Serialize)]
pub struct UpdateTaskConfigRequest {
    /// New priority, if changing.
    pub priority: Option<i32>,
    /// New worker pool, if changing.
    pub pool: Option<String>,
    /// New retry policy, if changing.
    pub retry_policy: Option<RetryPolicy>,
    /// New rate-limit policy, if changing.
    pub rate_limit_policy: Option<RateLimitPolicy>,
}
