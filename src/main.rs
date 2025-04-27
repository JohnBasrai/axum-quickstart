use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};
use tracing_subscriber;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct Movie {
    id: String,
    title: String,
    year: u16,
    stars: f32,
}

use axum::response::{IntoResponse, Response};

/// Wrapper type for successful API responses.
///
/// Encapsulates the data payload and prepares it for JSON serialization.
#[derive(Serialize)]
struct ApiResponse<T> {
    data: T,
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

/// Handler for fetching a movie entry by ID (GET /get/{id}).
///
/// Looks up a movie by its unique ID in the database.
///
/// - If the movie exists, responds with `200 OK` and the full `Movie` object as JSON.
/// - If the movie does not exist, responds with `404 Not Found` and an empty body.
///
/// This endpoint enforces correct HTTP semantics for missing resources.
#[tracing::instrument(skip(state, id))]
async fn get_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, ApiResponse<Movie>), StatusCode> {
    // ---
    let mut conn = state.get_conn().await?;

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

    Ok((StatusCode::OK, ApiResponse { data: movie }))
}

async fn save_movie(
    conn: &mut redis::aio::MultiplexedConnection,
    movie_id: &str,
    movie: &Movie,
    allow_overwrite: bool,
) -> Result<StatusCode, StatusCode> {
    // ---
    if !allow_overwrite {
        let exists: bool = conn
            .exists(movie_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if exists {
            return Err(StatusCode::CONFLICT);
        }
    }

    let _: () = conn
        .hset_multiple(
            movie_id,
            &[
                ("id", &movie.id),
                ("title", &movie.title),
                ("year", &movie.year.to_string()),
                ("stars", &movie.stars.to_string()),
            ],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if allow_overwrite {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::CREATED)
    }
}

/// Handler for creating a new movie entry (POST /add).
///
/// Expects a complete `Movie` object in the request body.
///
/// - If the movie ID already exists in the database, responds with `409 Conflict`.
/// - On success, responds with `201 Created`.
///
/// This endpoint enforces uniqueness of movie IDs.
#[tracing::instrument(skip(state, movie))]
async fn add_movie(
    State(state): State<AppState>,
    Json(movie): Json<Movie>,
) -> Result<StatusCode, StatusCode> {
    // ---
    let mut conn = state.get_conn().await?;

    save_movie(&mut conn, &movie.id, &movie, false).await
}

/// Handler for updating an existing movie entry (PUT /update/{id}).
///
/// Expects a complete `Movie` object in the request body.
///
/// - Always overwrites any existing movie with the provided ID.
/// - Responds with `200 OK` regardless of whether the movie previously existed.
///
/// This endpoint allows overwriting or creating movies freely.
#[tracing::instrument(skip(state, updated_movie))]
async fn update_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(updated_movie): Json<Movie>,
) -> Result<StatusCode, StatusCode> {
    // ---
    let mut conn = state.get_conn().await?;

    save_movie(&mut conn, &id, &updated_movie, true).await
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
        .route("/get/{id}", get(get_movie))
        .route("/add", post(add_movie))
        .route("/update/{id}", put(update_movie))
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
