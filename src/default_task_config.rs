use std::sync::Arc;

use chrono::Utc;

use boson_core::{RateLimitPolicy, RetryPolicy, TaskConfig};
use boson_runtime::TaskRegistry;

use crate::BosonCoordinatorBackend;

/// Upsert `boson_task_config` rows for every task discovered via `#[boson::task]` inventory.
///
/// Safe to call on every boot: existing rows are overwritten with the descriptor's current
/// defaults (priority, pool, retry/rate-limit policy, idempotency mode), so this keeps
/// `boson_task_config` in sync as tasks are added, removed, or have their `#[boson::task]`
/// attributes changed.
///
/// # Examples
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// use boson_coordinator::{ensure_default_task_configs_embedded, BosonCoordinatorBackend};
///
/// # async fn boot(backend: Arc<dyn BosonCoordinatorBackend>) -> anyhow::Result<()> {
/// ensure_default_task_configs_embedded(backend).await?;
/// # Ok(())
/// # }
/// ```
pub async fn ensure_default_task_configs_embedded(
    backend: Arc<dyn BosonCoordinatorBackend>,
) -> anyhow::Result<()> {
    ensure_default_task_configs_embedded_with_skip(backend, &[]).await
}

/// Like [`ensure_default_task_configs_embedded`], but skips task names listed in `skip`.
///
/// Use this when a host manages some tasks' config by hand (e.g. an admin UI has already set
/// custom priority/retry policy) and does not want this bootstrap to overwrite it.
pub async fn ensure_default_task_configs_embedded_with_skip(
    backend: Arc<dyn BosonCoordinatorBackend>,
    skip: &[&str],
) -> anyhow::Result<()> {
    let registry = TaskRegistry::auto_discover();
    for name in registry.sorted_task_names() {
        if skip.contains(&name) {
            continue;
        }
        let Some(d) = registry.get(name) else {
            continue;
        };
        let cfg = TaskConfig {
            task_name: d.name.to_string(),
            priority: d.default_priority,
            pool: d.default_pool.to_string(),
            retry_policy: RetryPolicy {
                max_attempts: d.default_retry_max_attempts,
                base_delay_ms: d.default_retry_base_delay_ms,
                backoff_multiplier: d.default_retry_backoff_multiplier,
                max_delay_ms: d.default_retry_max_delay_ms,
            },
            rate_limit_policy: RateLimitPolicy {
                max_in_flight: d.default_rate_max_in_flight,
                max_enqueue_per_second: d.default_rate_max_enqueue_per_second,
            },
            idempotency_mode: d.default_idempotency_mode,
            updated_at: Utc::now(),
        };
        backend
            .upsert_task_config(cfg)
            .await
            .map_err(|e| anyhow::anyhow!("ensure boson task config {}: {}", d.name, e))?;
    }
    Ok(())
}
