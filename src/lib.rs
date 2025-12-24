// src/lib.rs
use anyhow::Result;
use app_state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

use handlers::health::health_check;
use handlers::metrics::metrics_handler;
use handlers::movies::*;
use handlers::root::root_handler;
use redis::Client;
use std::env;

// Public exports (visible outside this module)
pub mod domain;

// Internal-only exports (sibling access within this module)
mod app_state;
mod handlers;
mod infrastructure;

// Publicly expose the infrastructure creation functions
pub use infrastructure::{create_noop_metrics, create_postgres_repository, create_prom_metrics};

/// Build the HTTP router with metrics implementation determined by environment variables.
pub fn create_router() -> Result<Router> {
    // ---
    // Determine metrics implementation from environment
    let metrics_type = env::var("AXUM_METRICS_TYPE").unwrap_or_else(|_| "noop".to_string());

    let metrics = if metrics_type == "prom" {
        create_prom_metrics()?
    } else {
        create_noop_metrics()?
    };

    tracing_subscriber::fmt::try_init().ok(); // ✅ Ignores if already initialized

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = Client::open(redis_url)?;
    let app_state = AppState::new(redis_client, metrics);

    let router = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .nest(
            "/movies",
            Router::new()
                .route("/get/{id}", get(get_movie))
                .route("/add", post(add_movie))
                .route("/update/{id}", put(update_movie))
                .route("/delete/{id}", delete(delete_movie)),
        )
        .with_state(app_state);

    Ok(router)
}
