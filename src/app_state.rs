//! Application state management.
//!
//! This module defines the shared state structure that gets passed to all
//! Axum handlers via the `State` extractor. The `AppState` contains shared
//! resources like database connections, metrics implementations, and other
//! application-wide dependencies.
//!
//! The state is designed to be cheaply cloneable (using `Arc` internally
//! where needed) so it can be passed efficiently to each request handler
//! without expensive copying of resources.

use crate::domain::MetricsPtr;
use axum::http::StatusCode;
use redis::Client;

/// Shared application state passed to all Axum handlers.
///
/// Currently holds a Redis `Client` instance for creating multiplexed
/// async connections on demand inside each handler, and a metrics
/// implementation for recording application events.
///
/// This design allows each request to create an independent
/// connection safely while sharing the underlying Redis configuration
/// and metrics collection.
///
/// Additional shared resources (e.g., configuration, database pools)
/// can be added to this struct in the future as needed.
#[derive(Clone)]
pub(crate) struct AppState {
    redis_client: Client,
    metrics: MetricsPtr,
}

impl AppState {
    // ---

    pub fn new(redis_client: Client, metrics: MetricsPtr) -> Self {
        // ---
        AppState {
            redis_client,
            metrics,
        }
    }

    /// Creates a new multiplexed Redis connection.
    ///
    /// Logs an error if connection fails and returns HTTP 500.
    pub(crate) async fn get_conn(&self) -> Result<redis::aio::MultiplexedConnection, StatusCode> {
        // ---

        self.redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                tracing::error!("Failed to connect to Redis: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    /// Get a reference to the metrics implementation.
    pub(crate) fn metrics(&self) -> &MetricsPtr {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    // ---

    use super::*;
    use crate::infrastructure::create_noop_metrics;

    #[test]
    fn test_app_state_creation_and_clone() {
        // Test basic creation and that Clone works
        let redis_client = Client::open("redis://127.0.0.1:6379").unwrap();
        let metrics = create_noop_metrics().unwrap();

        let app_state = AppState::new(redis_client, metrics);
        let _cloned = app_state.clone();

        // Verify accessor works
        let _metrics_ref = app_state.metrics();
    }

    #[tokio::test]
    async fn test_redis_connection_failure() {
        // Test that connection failures return proper error
        let redis_client = Client::open("redis://invalid-host:6379").unwrap();
        let metrics = create_noop_metrics().unwrap();

        let app_state = AppState::new(redis_client, metrics);

        let result = app_state.get_conn().await;
        assert_eq!(result.unwrap_err(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
