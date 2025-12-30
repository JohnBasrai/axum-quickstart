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
mod config;
mod handlers;
mod infrastructure;
mod session;

// Hoist up only the public symbol(s)
pub use session::{create_session, validate_session, SessionInfo};

pub use config::*;

// Publicly expose the infrastructure creation functions
pub use infrastructure::{
    create_noop_metrics, // ---
    create_postgres_repository,
    create_prom_metrics,
    create_webauthn,
};

/// Build the HTTP router with metrics implementation determined by environment variables.
pub fn create_router() -> Result<Router> {
    // ---
    // Load all configuration from environment
    let config = AppConfig::from_env()?;

    // Determine metrics implementation from environment
    let metrics_type = env::var("AXUM_METRICS_TYPE").unwrap_or_else(|_| "noop".to_string());
    let metrics = if metrics_type == "prom" {
        create_prom_metrics()?
    } else {
        create_noop_metrics()?
    };

    tracing_subscriber::fmt::try_init().ok(); // ✅ Ignores if already initialized

    // Create infrastructure dependencies
    let redis_client = Client::open(config.redis.url.clone())?;
    let repository = create_postgres_repository()?;
    let webauthn = std::sync::Arc::new(create_webauthn(&config.webauthn)?);

    // Build application state with all dependencies
    let app_state = AppState::new(
        redis_client,
        metrics,
        repository,
        webauthn,
        config.redis.webauthn_challenge_ttl,
    );

    // Build router (Phase 2 WebAuthn routes will be added next)
    //
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
        .nest(
            "/webauthn",
            Router::new()
                .route(
                    "/register/start",
                    post(handlers::webauthn_register::register_start),
                )
                .route(
                    "/register/finish",
                    post(handlers::webauthn_register::register_finish),
                )
                .route(
                    "/auth/start",
                    post(handlers::webauthn_authenticate::auth_start),
                )
                .route(
                    "/auth/finish",
                    post(handlers::webauthn_authenticate::auth_finish),
                )
                .route(
                    "/credentials",
                    get(handlers::webauthn_credentials::list_credentials),
                )
                .route(
                    "/credentials/{id}",
                    delete(handlers::webauthn_credentials::delete_credential),
                ),
        )
        .with_state(app_state);

    Ok(router)
}
