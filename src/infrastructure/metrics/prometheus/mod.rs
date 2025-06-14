mod counters;
mod prometheus_metrics;
mod recorder;

pub use prometheus_metrics::PrometheusMetrics;
use std::sync::Arc;

// Re-export utilities for internal use within this module
pub(crate) use counters::{increment_movie_created, track_http_request};
pub(crate) use recorder::{init_metrics, render_metrics};

/// Creates a new Prometheus metrics implementation.
///
/// This implementation collects metrics in Prometheus format and can
/// expose them via HTTP endpoint for scraping.
///
/// Returns a fully initialized metrics instance ready for use.
pub fn create() -> anyhow::Result<crate::domain::MetricsPtr> {
    tracing::info!("Initializing Prometheus metrics");
    // TODO: Start HTTP server for /metrics endpoint, initialize registry, etc.
    init_metrics();

    Ok(Arc::new(PrometheusMetrics::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_returns_valid_metrics() {
        let result = create();
        assert!(result.is_ok());
    }
}
