use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode
};
use crate::handlers::shared_types::ApiResponse;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Movie {
    id: String,
    title: String,
    year: u16,
    stars: f32,
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
pub async fn get_movie(
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
pub async fn add_movie(
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
pub async fn update_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(updated_movie): Json<Movie>,
) -> Result<StatusCode, StatusCode> {
    // ---
    let mut conn = state.get_conn().await?;

    save_movie(&mut conn, &id, &updated_movie, true).await
}

/// Delete a movie from the Redis database by its ID.
///
/// Returns:
/// - `204 No Content` if the movie was successfully deleted.
/// - `404 Not Found` if no movie exists with the given ID.
/// - `500 Internal Server Error` if there is a Redis connection or command failure.
///
/// # Arguments
/// - `State(state)`: The application state, providing a Redis connection.
/// - `Path(id)`: The ID of the movie to delete.
///
/// # Errors
/// Returns a `StatusCode` error on failure, following the rules above.
pub async fn delete_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // ---
    let mut conn = state.get_conn().await?;

    let deleted: u64 = conn
        .del(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

