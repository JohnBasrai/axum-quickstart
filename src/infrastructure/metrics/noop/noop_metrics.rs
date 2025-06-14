use crate::domain::Metrics;
use std::time::Instant;

/// No-op metrics implementation for testing.
pub struct NoopMetrics;

impl NoopMetrics {
    pub fn new() -> Self {
        NoopMetrics
    }
}

impl Metrics for NoopMetrics {
    // ---
    fn render(&self) -> String {
        String::new()
    }
    fn record_movie_created(&self) {}
    fn record_http_request(&self, _: Instant, _: &str, _: &str, _: u16) {}
}
