use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::info;
use tracing_subscriber;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct Movie {
    id: String,
    title: String,
    year: u16,
    stars: f32,
}

// Lookup a movie by ID, suppress default tracing input parameters
#[tracing::instrument(skip(state, id))]
async fn get_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Movie>), StatusCode> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let fields: Vec<(String, String)> = conn
        .hgetall(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if fields.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let map: std::collections::HashMap<String, String> = fields.into_iter().collect();
    let movie = Movie {
        id: map.get("id").cloned().unwrap_or_default(),
        title: map.get("title").cloned().unwrap_or_default(),
        year: map.get("year").and_then(|y| y.parse().ok()).unwrap_or(0),
        stars: map.get("stars").and_then(|s| s.parse().ok()).unwrap_or(0.0),
    };

    Ok((StatusCode::OK, Json(movie)))
}

#[tracing::instrument(skip(state, movie))]
async fn add_movie(
    State(state): State<AppState>,
    Json(movie): Json<Movie>,
) -> Result<StatusCode, StatusCode> {
    info!("{}/{}", &movie.id, &movie.title);

    // Get redis handle and save the movie record under the key.
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _: () = conn
        .hset_multiple(
            &movie.id,
            &[
                ("id", &movie.id),
                ("title", &movie.title),
                ("year", &movie.year.to_string()),
                ("stars", &movie.stars.to_string()),
            ],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber to log to stdout
    tracing_subscriber::fmt::init();
    tracing::info!("Starting axum server...");

    // Get REDIS_URL from environment, or fallback to localhost.
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Create a Redis client. Multiplexed connections will be created on demand.
    let redis_client = Client::open(redis_url)?;

    let app = Router::new()
        .route("/get/{id}", get(get_movie))
        .route("/add", post(add_movie))
        .route(
            "/",
            get(|| async {
                r#"Welcome to the Movie API 👋

Available endpoints:
  - GET  /get/{id}  (Fetch a movie by ID)
  - POST /add       (Add a movie entry)

This script demonstrates successful adds, fetches, and 404 behavior for missing entries.
"#
            }),
        )
        .with_state(AppState { redis_client });

    // Get optional bind endpoint from environment
    let endpoint = env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    info!("Starting at endpoint:{}", endpoint);
    info!("Starting Axum Quick-Start API server v{}...", env!("CARGO_PKG_VERSION"));

    let listener = tokio::net::TcpListener::bind(&endpoint).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
