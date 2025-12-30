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

/// Validated session information extracted from Redis.
///
/// This struct is returned after successful session token validation
/// and contains the authenticated user's details.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    // ---
    pub user_id: Uuid,
    pub username: String,
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

// ---

/// Validates a session token and returns the authenticated user's information.
///
/// Extracts the session token from Redis and verifies it hasn't expired.
///
/// # Security
///
/// - Validates token exists in Redis (stateful session management)
/// - Checks expiration timestamp
/// - Returns user_id for authorization checks
///
/// # Arguments
/// * `redis_conn` - Active Redis connection
/// * `token` - Session token (typically from Authorization header)
///
/// # Returns
/// SessionInfo on success, or HTTP status code on failure
///
/// # Errors
///
/// Returns an error if:
/// - Token is not found in Redis (expired or invalid)
/// - Session data cannot be deserialized
/// - Session has expired
pub async fn validate_session(
    redis_conn: &mut MultiplexedConnection,
    token: &str,
) -> Result<SessionInfo, StatusCode> {
    // ---
    // format!() allocates ~40-50 bytes on heap per request.
    // In a hot path this contributes to allocator contention, but
    // Redis I/O (1-5ms) and JSON parsing (dozens of allocations)
    // dominate request latency. Optimize those first.
    let redis_key = format!("session:{token}");

    // Fetch session data from Redis
    let session_json: Option<String> = redis_conn.get(&redis_key).await.map_err(|e| {
        // ---
        tracing::error!("Failed to query Redis for session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let session_json = session_json.ok_or_else(|| {
        // ---
        tracing::debug!("Session token not found or expired: {}", token);
        StatusCode::UNAUTHORIZED
    })?;

    // Deserialize session data
    let session_data: SessionData = serde_json::from_str(&session_json).map_err(|e| {
        // ---
        tracing::error!("Failed to deserialize session data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if session has expired
    let now = chrono::Utc::now().timestamp();
    if session_data.expires_at < now {
        // ---
        tracing::debug!("Session expired for user: {}", session_data.username);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Parse user_id from string
    let user_id = Uuid::parse_str(&session_data.user_id).map_err(|e| {
        // ---
        tracing::error!("Invalid user_id in session data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(SessionInfo {
        user_id,
        username: session_data.username,
    })
}
