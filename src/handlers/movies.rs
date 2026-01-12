use super::ApiResponse;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{Datelike, Utc};
use redis::AsyncCommands;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::time::Instant;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Movie {
    title: String,
    year: u16,
    stars: f32,
}

#[derive(Debug, Clone)]
pub struct HashKey {
    pub value: String,
}

impl Movie {
    // ---

    /// Sanitizes the Movie instance by trimming whitespace,
    /// collapsing multiple spaces, validating fields, and generating
    /// a HashKey based on normalized title and year.
    pub fn sanitize(&mut self) -> Result<HashKey, StatusCode> {
        // ---

        let re = Regex::new(r"\s+").unwrap();

        // Trim leading/trailing and collapse internal spaces
        let trimmed = self.title.trim();
        let squeezed = re.replace_all(trimmed, " ");
        self.title = squeezed.to_string();

        // Validation
        if self.title.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        let current_year = Utc::now().year() as u16;
        if self.year < 1880 || self.year > current_year + 5 {
            return Err(StatusCode::BAD_REQUEST);
        }

        if !(0.0..=5.0).contains(&self.stars) {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Now generate the lookup key
        let combined = format!("{}:{}", self.title.to_lowercase(), self.year);
        let mut hasher = Sha1::new();
        hasher.update(combined.as_bytes());
        let result = hasher.finalize();
        let key = hex::encode(result);

        Ok(HashKey { value: key })
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
pub async fn get_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, ApiResponse<Movie>), StatusCode> {
    // ---

    let start = Instant::now();
    let mut conn = state.get_conn().await?;

    tracing::debug!("get movie: {id}");

    let result: Option<String> = conn.get(&id).await.map_err(|err| {
        tracing::info!("Got internal server error: {:?}", &err);
        state
            .metrics()
            .record_http_request(start, "/movies/get", "GET", 500);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let json_string = match result {
        Some(val) => val,
        None => {
            tracing::trace!("Movie not found: {id}");
            state
                .metrics()
                .record_http_request(start, "/movies/get", "GET", 404);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let movie: Movie = serde_json::from_str(&json_string).map_err(|err| {
        tracing::info!("Error parsing JSON: {:?}", &err);
        state
            .metrics()
            .record_http_request(start, "/movies/get", "GET", 400);
        StatusCode::BAD_REQUEST
    })?;

    tracing::trace!("Movie return: {}/{:?}", &id, &movie);
    state
        .metrics()
        .record_http_request(start, "/movies/get", "GET", 200);

    Ok((StatusCode::OK, ApiResponse { data: movie }))
}

async fn save_movie(
    conn: &mut redis::aio::MultiplexedConnection,
    movie_id: &str,
    movie: &Movie,
    allow_overwrite: bool,
) -> Result<StatusCode, StatusCode> {
    // ---

    tracing::trace!("save_movie {}/{:?}", &movie_id, &movie);

    if !allow_overwrite {
        let exists: bool = conn.exists(movie_id).await.map_err(|err| {
            tracing::info!("Got internal server error (1): {:?}", &err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        if exists {
            tracing::trace!("Conflict");
            return Err(StatusCode::CONFLICT);
        }
    }

    let movie_json = serde_json::to_string(movie).map_err(|err| {
        tracing::info!("Serialization error: {:?}", &err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    tracing::trace!("Writing movie: {:?}", &movie_json);

    let _: () = conn.set(movie_id, movie_json).await.map_err(|err| {
        tracing::info!("Got internal server error (2): {:?}", &err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::warn!("save movie OK");

    if allow_overwrite {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::CREATED)
    }
}

// Response for add_movie
#[derive(Serialize)]
pub struct CreatedResponse {
    id: String,
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
    Json(mut movie): Json<Movie>,
) -> Result<(StatusCode, Json<CreatedResponse>), StatusCode> {
    // ---

    let start = Instant::now();

    // Sanitize the movie and get a hash key for it
    let hash_key = movie.sanitize().inspect_err(|_err| {
        state
            .metrics()
            .record_http_request(start, "/movies/add", "POST", 400);
    })?;

    let mut conn = state.get_conn().await.map_err(|_| {
        state
            .metrics()
            .record_http_request(start, "/movies/add", "POST", 500);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let redis_key = hash_key.value;

    // Create a span with movie details for tracing
    let span = tracing::info_span!(
        "add_movie",
        title = %movie.title,
        year = movie.year,
        key = %redis_key
    );
    let _enter = span.enter();

    // Check if movie already exists
    if redis::cmd("EXISTS")
        .arg(&redis_key)
        .query_async::<i32>(&mut conn)
        .await
        .map_err(|_| {
            state
                .metrics()
                .record_http_request(start, "/movies/add", "POST", 500);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        != 0
    {
        tracing::debug!("Duplicate detected: {}", &redis_key);
        state
            .metrics()
            .record_http_request(start, "/movies/add", "POST", 409);
        return Err(StatusCode::CONFLICT);
    }

    tracing::debug!("Inserting new movie, key:{redis_key}");

    // Insert new movie
    let serialized = serde_json::to_string(&movie).map_err(|_| {
        state
            .metrics()
            .record_http_request(start, "/movies/add", "POST", 500);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    redis::cmd("SET")
        .arg(&redis_key)
        .arg(&serialized)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|_| {
            state
                .metrics()
                .record_http_request(start, "/movies/add", "POST", 500);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Record successful movie creation
    state.metrics().record_movie_created();
    state
        .metrics()
        .record_http_request(start, "/movies/add", "POST", 201);

    Ok((StatusCode::CREATED, Json(CreatedResponse { id: redis_key })))
}

/// Handler for updating an existing movie entry (PUT /update/{id}).
///
/// Expects a complete `Movie` object in the request body.
///
/// - Always overwrites any existing movie with the provided ID.
/// - Responds with `200 OK` regardless of whether the movie previously existed.
///
/// This endpoint allows overwriting or creating movies freely.
#[tracing::instrument(skip(state, movie))]
pub async fn update_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut movie): Json<Movie>,
) -> Result<StatusCode, StatusCode> {
    // ---

    let start = Instant::now();

    movie.sanitize().inspect_err(|_err| {
        state
            .metrics()
            .record_http_request(start, "/movies/update", "PUT", 400);
    })?;

    let mut conn = state.get_conn().await.inspect_err(|_err| {
        state
            .metrics()
            .record_http_request(start, "/movies/update", "PUT", 500);
    })?;

    let result = save_movie(&mut conn, &id, &movie, true).await;

    match &result {
        Ok(status) => {
            state
                .metrics()
                .record_http_request(start, "/movies/update", "PUT", status.as_u16());
        }
        Err(status) => {
            state
                .metrics()
                .record_http_request(start, "/movies/update", "PUT", status.as_u16());
        }
    }

    result
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

    let start = Instant::now();

    let mut conn = state.get_conn().await.inspect_err(|_err| {
        state
            .metrics()
            .record_http_request(start, "/movies/delete", "DELETE", 500);
    })?;

    let deleted: u64 = conn.del(&id).await.map_err(|_| {
        state
            .metrics()
            .record_http_request(start, "/movies/delete", "DELETE", 500);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if deleted == 0 {
        state
            .metrics()
            .record_http_request(start, "/movies/delete", "DELETE", 404);
        Err(StatusCode::NOT_FOUND)
    } else {
        state
            .metrics()
            .record_http_request(start, "/movies/delete", "DELETE", 204);
        Ok(StatusCode::NO_CONTENT)
    }
}

#[cfg(test)]
mod tests {
    // ---

    use super::*;
    use axum::http::StatusCode;

    fn sanitize_ok(title: &str, year: u16, stars: f32) -> HashKey {
        let mut movie = Movie {
            title: title.to_string(),
            year,
            stars,
        };
        movie.sanitize().expect("Expected sanitize to succeed")
    }

    fn sanitize_err(title: &str, year: u16, stars: f32) -> StatusCode {
        let mut movie = Movie {
            title: title.to_string(),
            year,
            stars,
        };
        movie.sanitize().unwrap_err()
    }

    #[test]
    fn test_normal_title_sanitization() {
        let key = sanitize_ok("The Shawshank Redemption", 1994, 4.5);
        assert_eq!(key.value.len(), 40); // SHA1 hex = 40 characters
    }

    #[test]
    fn test_title_with_extra_spaces() {
        let key = sanitize_ok(" The    Shawshank    Redemption ", 1994, 4.5);
        let key2 = sanitize_ok("The Shawshank Redemption", 1994, 4.5);
        assert_eq!(
            key.value, key2.value,
            "Key must be the same after collapsing spaces"
        );
    }

    #[test]
    fn test_title_mixed_case() {
        let key = sanitize_ok("The SHAWshank Redemption", 1994, 4.5);
        let key2 = sanitize_ok("the shawshank redemption", 1994, 4.5);
        assert_eq!(key.value, key2.value, "Key must be case-insensitive");
    }

    #[test]
    fn test_empty_title_rejected() {
        let status = sanitize_err("   ", 1994, 4.5);
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_bad_year_rejected() {
        let status = sanitize_err("Test Movie", 1700, 4.5);
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let future_year = (chrono::Utc::now().year() as u16) + 10;
        let status = sanitize_err("Test Movie", future_year, 4.5);
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_invalid_stars_rejected() {
        let status = sanitize_err("Test Movie", 1994, -1.0);
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let status = sanitize_err("Test Movie", 1994, 6.0);
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
