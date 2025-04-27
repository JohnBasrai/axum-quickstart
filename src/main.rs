use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Movie {
    id: String,
    title: String,
    year: u16,
    stars: f32,
}

#[derive(Serialize)]
struct GetMovie {
    // Will be None if a Movie is not found under the lookup key.
    movie: Option<Movie>,
}

// The movie database is made thread-safe using Arc<Mutex<_>>.
// Added as a state object (.with_state(db)) which AXUM will pass to each handler.
// This approach is appropriate for light to moderate write contention.
// For heavy concurrent writes, consider sharding the database to improve performance.
type DB = Arc<Mutex<HashMap<String, Movie>>>;

// Lookup a movie by ID, suppress default tracing input parameters
#[tracing::instrument(skip(db, id))]
async fn get_movie(Path(id): Path<String>, State(db): State<DB>) -> (StatusCode, Json<GetMovie>) {
    // Lock the database for safe concurrent access; the lock is released when db_guard is dropped
    // Note: lock().await is infallible in tokio::sync::Mutex (no poison errors)
    let db_guard = db.lock().await;

    let (status, movie) = if let Some(movie) = db_guard.get(&id) {
        info!("{}/{}", &movie.id, &movie.title);
        (StatusCode::OK, Some(movie.clone()))
    } else {
        info!("{}/<NOT-FOUND>", &id);
        (StatusCode::NOT_FOUND, None)
    };
    let movie = GetMovie { movie };
    (status, Json(movie))
}

#[tracing::instrument(skip(db, movie))]
async fn add_movie(State(db): State<DB>, Json(movie): Json<Movie>) -> StatusCode {
    info!("{}/{}", &movie.id, &movie.title);

    // Lock the database for safe concurrent access; the lock is released when db_guard is dropped
    let mut db_guard = db.lock().await;

    db_guard.insert(movie.id.clone(), movie);

    StatusCode::CREATED
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber to log to stdout (can be customized to log to a file or other formats)
    tracing_subscriber::fmt::init();
    tracing::info!("Starting axum server...");

    let db: DB = Arc::new(Mutex::new(HashMap::new()));

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
        .with_state(db);

    // Get optional bind endpoint from environment
    let endpoint = env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    info!("Starting at endpoint:{}", endpoint);
    info!("Starting Axum Quick-Start API server v{}...", env!("CARGO_PKG_VERSION"));

    let listener = tokio::net::TcpListener::bind(&endpoint).await?;
    axum::serve(listener, app).await.map_err(Into::into)
}
