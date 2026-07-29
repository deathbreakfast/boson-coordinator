//! Pure autoscaling math for Boson worker pools (queue-depth driven).
//!
//! # Examples
//!
//! ```rust
//! use boson_coordinator::scaling::{compute_target_workers, AutoscalePolicy};
//!
//! let policy = AutoscalePolicy::default();
//! let target = compute_target_workers(25, 2, &policy);
//! assert!(target >= policy.min_workers);
//! ```

/// Policy with hysteresis: scale up when the queue is deep per worker; scale down when idle.
///
/// # Examples
///
/// ```rust
/// use boson_coordinator::scaling::AutoscalePolicy;
///
/// let policy = AutoscalePolicy {
///     min_workers: 1,
///     max_workers: 20,
///     scale_up_queue_per_worker: 10,
///     scale_down_queue_per_worker: 2,
/// };
/// assert_eq!(policy.min_workers, 1);
/// ```
#[derive(Debug, Clone)]
pub struct AutoscalePolicy {
    /// Never scale below this many workers, even when the queue is empty.
    pub min_workers: u32,
    /// Never scale above this many workers, even when the queue is very deep.
    pub max_workers: u32,
    /// If `queue_depth > current_workers * scale_up_queue_per_worker`, add one worker.
    pub scale_up_queue_per_worker: u32,
    /// If `queue_depth <= (current_workers - 1) * scale_down_queue_per_worker`, remove one worker.
    pub scale_down_queue_per_worker: u32,
}

impl Default for AutoscalePolicy {
    fn default() -> Self {
        Self {
            min_workers: 1,
            max_workers: 64,
            scale_up_queue_per_worker: 10,
            scale_down_queue_per_worker: 2,
        }
    }
}

/// Returns a new desired worker count given global queue depth and current allocation.
///
/// Moves by at most one worker per call — call this periodically (e.g. every autoscale tick)
/// rather than trying to jump straight to a "correct" count, so scaling stays gradual and the
/// hysteresis in [`AutoscalePolicy`] can prevent flapping.
///
/// # Examples
///
/// ```rust
/// use boson_coordinator::scaling::{compute_target_workers, AutoscalePolicy};
///
/// let policy = AutoscalePolicy {
///     min_workers: 1,
///     max_workers: 10,
///     scale_up_queue_per_worker: 5,
///     scale_down_queue_per_worker: 1,
/// };
/// // Queue is deep relative to 3 workers (30 > 3 * 5) — scale up by one.
/// assert_eq!(compute_target_workers(30, 3, &policy), 4);
/// ```
pub fn compute_target_workers(
    queue_depth: u32,
    current_workers: u32,
    policy: &AutoscalePolicy,
) -> u32 {
    let min_workers = policy.min_workers.min(policy.max_workers);
    let max_workers = policy.min_workers.max(policy.max_workers);
    let cur = current_workers.clamp(min_workers, max_workers);
    let mut next = cur;
    if queue_depth > cur.saturating_mul(policy.scale_up_queue_per_worker) && next < max_workers {
        next += 1;
    } else if cur > min_workers {
        let threshold = (cur - 1).saturating_mul(policy.scale_down_queue_per_worker);
        if queue_depth <= threshold {
            next -= 1;
        }
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_up_when_queue_deep() {
        let p = AutoscalePolicy {
            min_workers: 1,
            max_workers: 10,
            scale_up_queue_per_worker: 5,
            scale_down_queue_per_worker: 1,
        };
        assert_eq!(compute_target_workers(30, 3, &p), 4);
    }

    #[test]
    fn hysteresis_prevents_flap() {
        let p = AutoscalePolicy {
            min_workers: 2,
            max_workers: 10,
            scale_up_queue_per_worker: 10,
            scale_down_queue_per_worker: 3,
        };
        // 4 workers, depth 10 — no scale-up (10 <= 40); no scale-down (10 > (4-1)*3 = 9 is false… 10<=9 is false)
        assert_eq!(compute_target_workers(10, 4, &p), 4);
    }

    #[test]
    fn scales_down_when_mostly_idle() {
        let p = AutoscalePolicy {
            min_workers: 1,
            max_workers: 10,
            scale_up_queue_per_worker: 10,
            scale_down_queue_per_worker: 2,
        };
        assert_eq!(compute_target_workers(0, 4, &p), 3);
    }

    #[test]
    fn respects_min_max() {
        let p = AutoscalePolicy {
            min_workers: 2,
            max_workers: 3,
            scale_up_queue_per_worker: 1,
            scale_down_queue_per_worker: 1,
        };
        assert_eq!(compute_target_workers(1000, 3, &p), 3);
        assert_eq!(compute_target_workers(0, 2, &p), 2);
    }

    #[test]
    fn default_policy_clamps_out_of_range_current() {
        let p = AutoscalePolicy::default();
        // current below min is clamped before scale math; empty queue stays at min.
        assert_eq!(compute_target_workers(0, 0, &p), p.min_workers);
        // current above max is clamped; deep queue cannot exceed max.
        assert_eq!(compute_target_workers(u32::MAX, 10_000, &p), p.max_workers);
    }

    #[test]
    fn normalizes_inverted_worker_bounds() {
        let p = AutoscalePolicy {
            min_workers: 10,
            max_workers: 2,
            scale_up_queue_per_worker: 1,
            scale_down_queue_per_worker: 1,
        };
        assert_eq!(compute_target_workers(0, 0, &p), 2);
        assert_eq!(compute_target_workers(u32::MAX, 10, &p), 10);
    }
}
