//! Adapts upstream [`Boson`](boson_runtime::Boson) to [`BosonCoordinatorBackend`](crate::coordinator_trait::BosonCoordinatorBackend).

use std::sync::Arc;

use async_trait::async_trait;
use boson_core::{
    EnqueueTrust, Job, JobFilter, JobKey, JobStatus, Result, Run, RunFilter, RunKey, TaskConfig,
    TaskRunStats,
};
use boson_runtime::Boson;

use crate::BosonCoordinatorBackend;

/// Wraps upstream runtime for host / server-function enqueue and admin.
///
/// # Examples
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// use boson_coordinator::{BosonCoordinatorBackend, CoordinatorAdapter};
///
/// # fn wire(boson: Arc<boson_runtime::Boson>) {
/// let backend: Arc<dyn BosonCoordinatorBackend> = Arc::new(CoordinatorAdapter::new(boson));
/// # let _ = backend;
/// # }
/// ```
pub struct CoordinatorAdapter {
    inner: Arc<Boson>,
}

impl CoordinatorAdapter {
    /// Create adapter from shared runtime.
    pub const fn new(inner: Arc<Boson>) -> Self {
        Self { inner }
    }

    /// Underlying upstream runtime (Axum state).
    pub fn runtime(&self) -> Arc<Boson> {
        Arc::clone(&self.inner)
    }
}

#[async_trait]
impl BosonCoordinatorBackend for CoordinatorAdapter {
    async fn enqueue(
        &self,
        task_name: &str,
        actor_json: serde_json::Value,
        params_json: serde_json::Value,
        idempotency_key: Option<String>,
    ) -> Result<String> {
        self.inner
            .enqueue(task_name, actor_json, params_json, idempotency_key)
            .await
    }

    async fn get_job(&self, job_id: &str) -> Option<Job> {
        self.inner.get_job(job_id).await.ok().flatten()
    }

    async fn list_jobs(
        &self,
        status_filter: Option<JobStatus>,
        offset: usize,
        limit: usize,
    ) -> Vec<Job> {
        self.inner
            .list_jobs(status_filter, offset, limit)
            .await
            .unwrap_or_default()
    }

    async fn list_jobs_after(
        &self,
        filter: &JobFilter,
        after: Option<&JobKey>,
        limit: usize,
    ) -> Vec<Job> {
        self.inner
            .list_jobs_after(filter, after, limit)
            .await
            .unwrap_or_default()
    }

    async fn count_jobs_matching(&self, filter: &JobFilter) -> Result<u64> {
        self.inner.queue_backend().count_jobs_matching(filter).await
    }

    /// Retries through [`Boson::retry_job`] with [`EnqueueTrust::External`], so the runtime's
    /// actor policy checks `actor_json` as it does for HTTP enqueue.
    async fn retry_job(&self, job_id: &str, actor_json: serde_json::Value) -> Result<String> {
        self.inner
            .retry_job(job_id, actor_json, EnqueueTrust::External)
            .await
    }

    async fn drop_failed_job(&self, job_id: &str) -> Result<()> {
        self.inner.queue_backend().drop_failed_job(job_id).await
    }

    async fn cancel_job(&self, job_id: &str) -> Result<()> {
        self.inner.cancel_job(job_id).await
    }

    async fn get_task_config(&self, task_name: &str) -> Result<TaskConfig> {
        self.inner.get_task_config(task_name).await
    }

    async fn upsert_task_config(&self, config: TaskConfig) -> Result<()> {
        self.inner.upsert_task_config(config).await
    }

    async fn list_runs(
        &self,
        job_id_filter: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Vec<Run> {
        self.inner
            .list_runs(job_id_filter, offset, limit)
            .await
            .unwrap_or_default()
    }

    async fn list_runs_after(
        &self,
        filter: &RunFilter,
        after: Option<&RunKey>,
        limit: usize,
    ) -> Vec<Run> {
        self.inner
            .list_runs_after(filter, after, limit)
            .await
            .unwrap_or_default()
    }

    async fn count_runs_matching(&self, filter: &RunFilter) -> Result<u64> {
        self.inner.queue_backend().count_runs_matching(filter).await
    }

    async fn get_run(&self, run_id: &str) -> Option<Run> {
        self.inner.get_run(run_id).await.ok().flatten()
    }

    fn registry(&self) -> &boson_runtime::TaskRegistry {
        self.inner.registry()
    }

    async fn count_jobs(&self, status_filter: Option<JobStatus>) -> u64 {
        self.inner.count_jobs(status_filter).await.unwrap_or(0)
    }

    async fn count_runs(&self, job_id_filter: Option<&str>) -> u64 {
        self.inner.count_runs(job_id_filter).await.unwrap_or(0)
    }

    async fn count_runs_since(&self, since: chrono::DateTime<chrono::Utc>) -> u64 {
        self.inner.count_runs_since(since).await.unwrap_or(0)
    }

    async fn count_jobs_for_task(&self, task_name: &str, status: Option<JobStatus>) -> u64 {
        self.inner
            .count_jobs_for_task(task_name, status)
            .await
            .unwrap_or(0)
    }

    async fn task_run_stats(&self, task_name: &str) -> TaskRunStats {
        self.inner
            .task_run_stats(task_name)
            .await
            .unwrap_or(TaskRunStats {
                runs_total: 0,
                success_count: 0,
            })
    }

    fn as_boson_runtime(&self) -> Option<Arc<Boson>> {
        Some(Arc::clone(&self.inner))
    }
}
