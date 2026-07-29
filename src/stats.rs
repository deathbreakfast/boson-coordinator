//! Lightweight queue metrics for autoscaling hooks.
//!
//! Pair with [`crate::scaling`] — see [`count_queued_jobs`].
//!
//! # Examples
//!
//! ```rust,ignore
//! use boson_coordinator::stats::count_queued_jobs;
//! use boson_coordinator::scaling::{compute_target_workers, AutoscalePolicy};
//!
//! let depth = count_queued_jobs(backend).await;
//! let target = compute_target_workers(depth as u32, current_workers, &AutoscalePolicy::default());
//! let _ = target;
//! ```

use boson_core::JobStatus;

use crate::BosonCoordinatorBackend;

/// Count jobs currently in `queued` status via the coordinator backend.
///
/// Feed this into [`crate::scaling::compute_target_workers`] as `queue_depth` to drive
/// autoscaling from live queue state.
///
/// # Examples
///
/// ```rust,ignore
/// use boson_coordinator::stats::count_queued_jobs;
/// use boson_coordinator::scaling::{compute_target_workers, AutoscalePolicy};
///
/// # async fn tick(backend: &dyn boson_coordinator::BosonCoordinatorBackend, current_workers: u32) {
/// let depth = count_queued_jobs(backend).await;
/// let target = compute_target_workers(depth as u32, current_workers, &AutoscalePolicy::default());
/// # let _ = target;
/// # }
/// ```
pub async fn count_queued_jobs(backend: &dyn BosonCoordinatorBackend) -> u64 {
    backend.count_jobs(Some(JobStatus::Queued)).await
}
