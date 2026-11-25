//! Filtered listing, filtered counts, retry and drop through the real adapter on the mem backend.
//!
//! Test-only fixtures below are intentionally undocumented; this binary target is exempt from
//! the library's `missing_docs = "deny"` lint (see `Cargo.toml`).
#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use boson_backend_mem::MemQueueBackend;
use boson_coordinator::{
    BosonCoordinatorBackend, BosonError, CoordinatorAdapter, JobFilter, RunFilter,
};
use boson_core::{ExecutionContext, JobStatus, JsonExecutionContextFactory, Run, RunStatus};
use boson_runtime::Boson;
use serde_json::json;

// Unlimited so retries are never rate limited.
#[boson_macros::task(name = "admin_alpha", max_in_flight = 0, max_enqueue_per_second = 0)]
async fn admin_alpha(_ctx: Box<dyn ExecutionContext>) -> boson_core::Result<()> {
    Ok(())
}

#[boson_macros::task(name = "admin_beta", max_in_flight = 0, max_enqueue_per_second = 0)]
async fn admin_beta(_ctx: Box<dyn ExecutionContext>) -> boson_core::Result<()> {
    Ok(())
}

/// Fresh mem backend plus a runtime over it; returns the runtime and its adapter.
fn mem_adapter() -> (Arc<Boson>, Arc<dyn BosonCoordinatorBackend>) {
    let boson = Arc::new(
        Boson::builder()
            .queue_backend(Arc::new(MemQueueBackend::new()))
            .execution_context_factory(JsonExecutionContextFactory)
            .auto_registry()
            .build()
            .unwrap(),
    );
    let adapter: Arc<dyn BosonCoordinatorBackend> =
        Arc::new(CoordinatorAdapter::new(Arc::clone(&boson)));
    (boson, adapter)
}

async fn enqueue(backend: &dyn BosonCoordinatorBackend, task: &str) -> String {
    backend
        .enqueue(task, json!({}), json!({}), None)
        .await
        .expect("enqueue")
}

/// Mark a job failed directly in the queue backend, as a worker would after its last attempt.
async fn fail(boson: &Boson, job_id: &str) {
    let queue = boson.queue_backend();
    let mut job = queue.get_job(job_id).await.unwrap().expect("job");
    job.status = JobStatus::Failed;
    queue.upsert_job(&job).await.unwrap();
}

async fn add_run(boson: &Boson, job_id: &str, task: &str, status: RunStatus) -> Run {
    let mut run = Run::new(job_id, task, 1);
    run.status = status;
    boson.queue_backend().upsert_run(&run).await.unwrap();
    run
}

/// Two alpha jobs (one failed), one beta job failed.
async fn fixture(boson: &Boson, backend: &dyn BosonCoordinatorBackend) -> [String; 3] {
    let alpha_queued = enqueue(backend, "admin_alpha").await;
    let alpha_failed = enqueue(backend, "admin_alpha").await;
    let beta_failed = enqueue(backend, "admin_beta").await;
    fail(boson, &alpha_failed).await;
    fail(boson, &beta_failed).await;
    [alpha_queued, alpha_failed, beta_failed]
}

fn job_ids(jobs: &[boson_core::Job]) -> Vec<String> {
    jobs.iter().map(|j| j.job_id.clone()).collect()
}

#[tokio::test]
async fn adapter_list_jobs_after_applies_filter() {
    let (boson, backend) = mem_adapter();
    let [alpha_queued, alpha_failed, beta_failed] = fixture(&boson, backend.as_ref()).await;

    let failed = backend
        .list_jobs_after(&JobFilter::status(JobStatus::Failed), None, 10)
        .await;
    let mut got = job_ids(&failed);
    got.sort();
    let mut want = vec![alpha_failed.clone(), beta_failed];
    want.sort();
    assert_eq!(got, want);

    let alpha = JobFilter {
        task: Some("admin_alpha".into()),
        ..JobFilter::default()
    };
    let mut got = job_ids(&backend.list_jobs_after(&alpha, None, 10).await);
    got.sort();
    let mut want = vec![alpha_queued.clone(), alpha_failed.clone()];
    want.sort();
    assert_eq!(got, want);

    let both = JobFilter {
        statuses: vec![JobStatus::Queued, JobStatus::Failed],
        task: Some("admin_alpha".into()),
        id_match: Some(alpha_queued[alpha_queued.len() - 6..].to_string()),
        ..JobFilter::default()
    };
    assert_eq!(
        job_ids(&backend.list_jobs_after(&both, None, 10).await),
        vec![alpha_queued]
    );
}

#[tokio::test]
async fn adapter_list_runs_after_applies_filter() {
    let (boson, backend) = mem_adapter();
    let [_, alpha_failed, beta_failed] = fixture(&boson, backend.as_ref()).await;
    let alpha_run = add_run(&boson, &alpha_failed, "admin_alpha", RunStatus::Failed).await;
    add_run(&boson, &beta_failed, "admin_beta", RunStatus::Failed).await;
    add_run(&boson, &beta_failed, "admin_beta", RunStatus::Success).await;

    let alpha = backend
        .list_runs_after(&RunFilter::job(&alpha_failed), None, 10)
        .await;
    assert_eq!(alpha.len(), 1);
    assert_eq!(alpha[0].run_id, alpha_run.run_id);

    let beta_failed_runs = RunFilter {
        statuses: vec![RunStatus::Failed],
        task: Some("admin_beta".into()),
        ..RunFilter::default()
    };
    let got = backend.list_runs_after(&beta_failed_runs, None, 10).await;
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].job_id, beta_failed);
    assert_eq!(got[0].status, RunStatus::Failed);
}

#[tokio::test]
async fn adapter_count_jobs_matching_counts_filter() {
    let (boson, backend) = mem_adapter();
    fixture(&boson, backend.as_ref()).await;

    assert_eq!(
        backend
            .count_jobs_matching(&JobFilter::default())
            .await
            .unwrap(),
        3
    );
    assert_eq!(
        backend
            .count_jobs_matching(&JobFilter::status(JobStatus::Failed))
            .await
            .unwrap(),
        2
    );
    let alpha_failed = JobFilter {
        statuses: vec![JobStatus::Failed],
        task: Some("admin_alpha".into()),
        ..JobFilter::default()
    };
    assert_eq!(backend.count_jobs_matching(&alpha_failed).await.unwrap(), 1);
}

#[tokio::test]
async fn adapter_count_runs_matching_counts_filter() {
    let (boson, backend) = mem_adapter();
    let [_, alpha_failed, beta_failed] = fixture(&boson, backend.as_ref()).await;
    add_run(&boson, &alpha_failed, "admin_alpha", RunStatus::Failed).await;
    add_run(&boson, &beta_failed, "admin_beta", RunStatus::Failed).await;
    add_run(&boson, &beta_failed, "admin_beta", RunStatus::Success).await;

    assert_eq!(
        backend
            .count_runs_matching(&RunFilter::default())
            .await
            .unwrap(),
        3
    );
    assert_eq!(
        backend
            .count_runs_matching(&RunFilter::job(&beta_failed))
            .await
            .unwrap(),
        2
    );
    let failed = RunFilter {
        statuses: vec![RunStatus::Failed],
        ..RunFilter::default()
    };
    assert_eq!(backend.count_runs_matching(&failed).await.unwrap(), 2);
}

#[tokio::test]
async fn adapter_retry_job_enqueues_a_linked_copy() {
    let (boson, backend) = mem_adapter();
    let [alpha_queued, alpha_failed, _] = fixture(&boson, backend.as_ref()).await;

    let actor = json!({"user": "admin-1"});
    let new_id = backend
        .retry_job(&alpha_failed, actor.clone())
        .await
        .unwrap();
    assert_ne!(new_id, alpha_failed);
    let new_job = backend.get_job(&new_id).await.expect("retried job");
    assert_eq!(new_job.status, JobStatus::Queued);
    assert_eq!(new_job.task_name, "admin_alpha");
    assert_eq!(new_job.retried_from.as_deref(), Some(alpha_failed.as_str()));
    assert_eq!(new_job.actor_json, actor);
    let old = backend.get_job(&alpha_failed).await.expect("original job");
    assert_eq!(old.status, JobStatus::Failed);

    let err = backend
        .retry_job(&alpha_queued, json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, BosonError::InvalidConfig(_)), "{err}");
    let err = backend
        .retry_job("no-such-job", json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, BosonError::JobNotFound(_)), "{err}");
}

#[tokio::test]
async fn adapter_drop_failed_job_hides_it_from_default_filters() {
    let (boson, backend) = mem_adapter();
    let [alpha_queued, alpha_failed, _] = fixture(&boson, backend.as_ref()).await;

    backend.drop_failed_job(&alpha_failed).await.unwrap();
    let dropped = backend.get_job(&alpha_failed).await.expect("dropped job");
    assert!(dropped.dropped_at.is_some());

    let visible = job_ids(
        &backend
            .list_jobs_after(&JobFilter::default(), None, 10)
            .await,
    );
    assert!(!visible.contains(&alpha_failed));
    assert_eq!(
        backend
            .count_jobs_matching(&JobFilter::default())
            .await
            .unwrap(),
        2
    );
    let with_dropped = JobFilter {
        include_dropped: true,
        ..JobFilter::default()
    };
    assert!(
        job_ids(&backend.list_jobs_after(&with_dropped, None, 10).await).contains(&alpha_failed)
    );

    let err = backend.drop_failed_job(&alpha_queued).await.unwrap_err();
    assert!(matches!(err, BosonError::InvalidConfig(_)), "{err}");
    let err = backend.drop_failed_job("no-such-job").await.unwrap_err();
    assert!(matches!(err, BosonError::JobNotFound(_)), "{err}");
}
