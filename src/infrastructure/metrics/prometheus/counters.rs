use metrics::{counter, histogram};
use std::time::Instant;

/// Increment a counter for created movies.
pub fn increment_movie_created() {
    counter!("movies_created_total").increment(1);
}

/// Track HTTP request latency using a histogram.
pub fn track_http_request(start: Instant) {
    let elapsed = start.elapsed();
    histogram!("http_request_duration_seconds").record(elapsed);
}
