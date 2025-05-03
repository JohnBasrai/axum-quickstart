// src/lib.rs
use anyhow::Result;
use axum::{
    http::StatusCode,
    routing::{delete, get, post, put},
    Router,
};
use redis::Client;
use std::env;

mod handlers;

use handlers::health::health_check;
use handlers::movies::*;
use handlers::root::root_handler;
use tracing::error;

/// Shared application state passed to all Axum handlers.
///
/// Currently holds a Redis `Client` instance for creating multiplexed
/// async connections on demand inside each handler.
///
/// This design allows each request to create an independent
/// connection safely while sharing the underlying Redis configuration.
///
/// Additional shared resources (e.g., configuration, database pools)
/// can be added to this struct in the future as needed.
#[derive(Clone)]
struct AppState {
    redis_client: Client,
}

impl AppState {
    // ---
    /// Creates a new multiplexed Redis connection.
    ///
    /// Logs an error if connection fails and returns HTTP 500.
    pub async fn get_conn(&self) -> Result<redis::aio::MultiplexedConnection, StatusCode> {
        // ---
        self.redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!("Failed to connect to Redis: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }
}

pub fn create_app() -> Result<Router> {
    // ---
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = Client::open(redis_url)?;

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .nest(
            "/movies",
            Router::new()
                .route("/get/{id}", get(get_movie))
                .route("/add", post(add_movie))
                .route("/update/{id}", put(update_movie))
                .route("/delete/{id}", delete(delete_movie)),
        )
        .with_state(AppState { redis_client });

    Ok(app)
}
