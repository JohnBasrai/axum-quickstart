//! Prometheus metrics implementation.
//!
//! This module provides a concrete implementation of the `Metrics` trait using
//! the Prometheus metrics format. It delegates to utility functions in sibling
//! modules (`counters.rs`, `recorder.rs`) which handle the actual metrics
//! collection via the global `metrics` crate registry.
//!
//! The implementation follows a global registry pattern where metrics are
//! automatically registered when first used, and a single global handle
//! manages rendering all collected metrics in Prometheus text format.

use crate::domain::Metrics;
use std::time::Instant;

/// Prometheus-based metrics implementation.
///
/// This struct is intentionally empty because we use the global metrics registry
/// pattern via the `metrics` crate. All metrics are registered globally using
/// macros like `counter!()` and `histogram!()`, and the global PrometheusHandle
/// stored in `recorder.rs` manages the actual metrics collection and rendering.
pub struct PrometheusMetrics {
    // Empty - uses global metrics registry pattern
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        tracing::info!("Creating Prometheus metrics");
        PrometheusMetrics {}
    }
}

impl Metrics for PrometheusMetrics {
    fn render(&self) -> String {
        // Use the recorder utility to get actual metrics
        super::render_metrics()
    }

    fn record_movie_created(&self) {
        tracing::debug!("Recording movie created event");
        super::increment_movie_created();
    }

    fn record_http_request(&self, start: Instant, _path: &str, _method: &str, _status: u16) {
        tracing::debug!("Recording HTTP request duration");
        super::track_http_request(start);
    }
}
