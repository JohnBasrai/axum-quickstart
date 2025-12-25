//! Session management for authenticated users.
//!
//! Provides session token generation and storage in Redis with configurable TTL.

use axum::http::StatusCode;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---

/// Session data stored in Redis.
#[derive(Debug, Serialize, Deserialize)]
struct SessionData {
    //
    user_id: String,
    username: String,
    expires_at: i64,
}

// ---

/// Session token time-to-live in seconds (7 days).
const SESSION_TTL_SECONDS: i64 = 604_800;

// ---

/// Creates a new session token and stores it in Redis.
///
/// # Arguments
/// * `redis_conn` - Active Redis connection
/// * `user_id` - User's unique identifier
/// * `username` - User's username
///
/// # Returns
/// Session token (UUID) on success, or HTTP status code on failure
pub async fn create_session(
    redis_conn: &mut MultiplexedConnection,
    user_id: Uuid,
    username: String,
) -> Result<String, StatusCode> {
    //
    let token = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now().timestamp() + SESSION_TTL_SECONDS;

    let session_data = SessionData {
        //
        user_id: user_id.to_string(),
        username: username.clone(),
        expires_at,
    };

    let session_json = serde_json::to_string(&session_data).map_err(|e| {
        //
        tracing::error!("Failed to serialize session data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let redis_key = format!("session:{token}");

    redis_conn
        .set_ex::<_, _, ()>(&redis_key, session_json, SESSION_TTL_SECONDS as u64)
        .await
        .map_err(|e| {
            //
            tracing::error!("Failed to store session in Redis: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Created session for user: {}", username);

    Ok(token)
}
