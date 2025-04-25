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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Movie {
    id: String,
    title: String,
    year: u16,
    stars: f32,
}

#[derive(Serialize)]
struct GetMovie {
    movie: Option<Movie>,
}

// Database is made thread-safe using Arc<Mutex<_>>.
// This approach is appropriate for light to moderate write contention.
// For heavy concurrent writes, consider sharding the database to improve performance.
type DB = Arc<Mutex<HashMap<String, Movie>>>;

// Lookup a movie by ID
async fn get_movie(Path(id): Path<String>, State(db): State<DB>) -> (StatusCode, Json<GetMovie>) {
    // Lock the database mutex for MT access. When db_guard is dropped lock is released
    let db_guard = db.lock().await;

    let (status, movie) = if let Some(movie) = db_guard.get(&id) {
        println!("Movie found! {:?}", &movie);
        (StatusCode::OK, Some(movie.clone()))
    } else {
        println!("Movie not found for id={id}!");
        (StatusCode::NOT_FOUND, None)
    };
    let movie = GetMovie { movie };
    (status, Json(movie))
}

async fn add_movie(State(db): State<DB>, Json(movie): Json<Movie>) -> (StatusCode, Json<Movie>) {
    // Lock the database mutex for MT access. When db_guard is dropped lock is released
    let mut db_guard = db.lock().await;

    db_guard.insert(movie.id.clone(), movie.clone());

    println!("Movie added! {:?}", &movie);
    (axum::http::StatusCode::CREATED, Json(movie))
}

#[tokio::main]
async fn main() -> Result<()> {
    let db: DB = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/get/{id}", get(get_movie))
        .route("/add", post(add_movie))
        .route(
            "/",
            get(|| async {
                r#"Welcome to the Movie API 👋

Available endpoints:
  - POST /get       (Add a movie)
  - GET  /add/{id}  (Fetch a movie by ID)

To see it in action, run the included ./api-demo.sh script.
"#
            }),
        )
        .with_state(db);

    let addr = env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    println!("Server starting on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await.map_err(Into::into)
}
