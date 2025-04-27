use anyhow::Result;
use axum::{http::StatusCode, Router, routing::{get, delete, post, put},};
use redis::Client;
use std::env;
use tracing::{error, info};

mod handlers;

use handlers::movies::*;
use handlers::health::health_check;
use handlers::root::root_handler;

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

use tracing_subscriber::fmt::format::FmtSpan;

// Initialize tracing subscriber
fn init_tracing() {
    // ---
    let span_events = match env::var("AXUM_SPAN_EVENTS").as_deref() {
        Ok("full") => FmtSpan::FULL,              // ENTER, EXIT, CLOSE with timing
        Ok("enter_exit") => FmtSpan::ENTER | FmtSpan::EXIT, // Only ENTER and EXIT
        _ => FmtSpan::CLOSE,                       // Default: only CLOSE timing
    };

    tracing_subscriber::fmt()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(span_events)
        .compact()
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // ---
    // Initialize tracing subscriber to log to stdout
    init_tracing();

    tracing::info!("Starting axum server...");

    // Get REDIS_URL from environment, or fallback to localhost.
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Create a Redis client. Multiplexed connections will be created on demand.
    let redis_client = Client::open(redis_url)?;

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(root_handler))
        .nest("/movies", Router::new()
              .route("/get/{id}", get(get_movie))
              .route("/add", post(add_movie))
              .route("/update/{id}", put(update_movie))
              .route("/delete/{id}", delete(delete_movie)))
        .with_state(AppState { redis_client });

    // Get optional bind endpoint from environment
    let endpoint = env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    info!("Starting at endpoint:{}", endpoint);
    info!("Starting Axum Quick-Start API server v{}...", env!("CARGO_PKG_VERSION"));

    let listener = tokio::net::TcpListener::bind(&endpoint).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
