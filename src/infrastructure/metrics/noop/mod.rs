// src/infrastructure/noop/mod.rs
mod noop_metrics;

pub use noop_metrics::NoopMetrics;
use std::sync::Arc;

/// Creates a new no-op metrics implementation.
///
/// This implementation does nothing - all metrics calls are ignored.
/// Useful for development, testing, or when metrics are disabled.
///
/// Returns a fully initialized metrics instance ready for use.
pub fn create() -> anyhow::Result<crate::domain::MetricsPtr> {
    Ok(Arc::new(NoopMetrics::new()))
}
