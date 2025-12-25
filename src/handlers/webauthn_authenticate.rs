//! WebAuthn authentication handlers.
//!
//! Implements the two-phase passkey authentication flow:
//! 1. `auth_start` - Generate challenge and return credential request options
//! 2. `auth_finish` - Verify credential, update counter, and create session token

use crate::app_state::AppState;
use crate::session;
use axum::{extract::State, http::StatusCode, Json};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AuthStartRequest {
    //
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStartResponse {
    //
    pub options: RequestChallengeResponse,
}

#[derive(Debug, Deserialize)]
pub struct AuthFinishRequest {
    //
    pub username: String,
    pub credential: PublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub struct AuthFinishResponse {
    //
    pub session_token: String,
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    //
    pub error: String,
}

// ============================================================================
// Authentication Start Handler
// ============================================================================

/// Initiates WebAuthn authentication by generating a challenge.
///
/// # Flow
/// 1. Verify user exists in database
/// 2. Fetch user's registered credentials
/// 3. Generate authentication challenge using webauthn-rs
/// 4. Store challenge in Redis with 5-minute expiry
/// 5. Return challenge options to client
///
/// # Security
/// - Returns generic error if user not found (prevent username enumeration)
/// - Challenge expires after configured TTL (typically 5 minutes)
pub async fn auth_start(
    State(state): State<AppState>,
    Json(req): Json<AuthStartRequest>,
) -> Result<Json<AuthStartResponse>, (StatusCode, Json<ErrorResponse>)> {
    //
    // Get user from database
    let user = state
        .repository()
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| {
            //
            tracing::error!("Database error fetching user '{}': {:?}", req.username, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            //
            tracing::warn!(
                "Authentication attempt for non-existent user: {}",
                req.username
            );
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?;

    // Fetch user's credentials
    let credentials = state
        .repository()
        .get_credentials_by_user(user.id)
        .await
        .map_err(|e| {
            //
            tracing::error!(
                "Database error fetching credentials for user '{}': {:?}",
                req.username,
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
        })?;

    if credentials.is_empty() {
        //
        tracing::warn!("User '{}' has no registered credentials", req.username);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Authentication failed".to_string(),
            }),
        ));
    }

    // Convert stored credentials to webauthn-rs Passkey format
    let passkeys: Vec<Passkey> = credentials
        .iter()
        .filter_map(|cred| {
            //
            serde_json::from_slice(&cred.public_key)
                .map_err(|e| {
                    //
                    tracing::error!(
                        "Failed to deserialize passkey for credential {}: {:?}",
                        hex::encode(&cred.id),
                        e
                    );
                })
                .ok()
        })
        .collect();

    if passkeys.is_empty() {
        //
        tracing::error!(
            "User '{}' has credentials but all failed deserialization",
            req.username
        );
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
            }),
        ));
    }

    // Generate authentication challenge
    let (options, auth_state) = state
        .webauthn()
        .start_passkey_authentication(&passkeys)
        .map_err(|e| {
            //
            tracing::error!("Failed to generate auth challenge: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
        })?;

    // Serialize and store challenge in Redis
    let state_json = serde_json::to_vec(&auth_state).map_err(|e| {
        //
        tracing::error!("Failed to serialize auth state: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
            }),
        )
    })?;

    let redis_key = format!("webauthn:auth:{}", req.username);
    let ttl_seconds = state.challenge_ttl().as_secs();

    let mut conn = state.get_conn().await.map_err(|status| {
        //
        tracing::error!("Failed to get Redis connection");
        (
            status,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
            }),
        )
    })?;

    conn.set_ex::<_, _, ()>(&redis_key, state_json, ttl_seconds)
        .await
        .map_err(|e| {
            //
            tracing::error!("Failed to store auth challenge in Redis: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
        })?;

    tracing::info!("Generated auth challenge for user: {}", req.username);

    Ok(Json(AuthStartResponse { options }))
}

// ============================================================================
// Authentication Finish Handler
// ============================================================================

/// Completes WebAuthn authentication by verifying the credential.
///
/// # Flow
/// 1. Retrieve and delete challenge from Redis (atomic GETDEL)
/// 2. Verify credential signature using webauthn-rs
/// 3. Validate counter prevents replay attacks
/// 4. Update counter in database
/// 5. Create session token and store in Redis
/// 6. Return session token to client
///
/// # Security
/// - Challenge automatically expires after TTL
/// - Counter must increment (prevents replay attacks)
/// - Returns generic error for all failures (no information leakage)
pub async fn auth_finish(
    State(state): State<AppState>,
    Json(req): Json<AuthFinishRequest>,
) -> Result<Json<AuthFinishResponse>, (StatusCode, Json<ErrorResponse>)> {
    //
    // Atomically retrieve and delete challenge from Redis
    let redis_key = format!("webauthn:auth:{}", req.username);

    let mut conn = state.get_conn().await.map_err(|status| {
        //
        tracing::error!("Failed to get Redis connection");
        (
            status,
            Json(ErrorResponse {
                error: "Authentication failed".to_string(),
            }),
        )
    })?;

    let state_bytes: Vec<u8> = conn.get_del(&redis_key).await.map_err(|e| {
        //
        tracing::warn!("Challenge not found or expired for user: {}", req.username);
        tracing::debug!("Redis error: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Challenge not found or expired".to_string(),
            }),
        )
    })?;

    // Deserialize challenge state
    let auth_state: PasskeyAuthentication = serde_json::from_slice(&state_bytes).map_err(|e| {
        //
        tracing::error!("Failed to deserialize auth state: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Authentication failed".to_string(),
            }),
        )
    })?;

    // Verify the credential using webauthn-rs
    let auth_result = state
        .webauthn()
        .finish_passkey_authentication(&req.credential, &auth_state)
        .map_err(|e| {
            //
            tracing::warn!(
                "Authentication verification failed for user '{}': {:?}",
                req.username,
                e
            );
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?;

    // Fetch the stored credential to validate counter
    let credential_id = auth_result.cred_id().to_vec();
    let mut stored_credential = state
        .repository()
        .get_credential_by_id(&credential_id)
        .await
        .map_err(|e| {
            //
            tracing::error!("Database error fetching credential: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            //
            tracing::error!(
                "Credential not found in database: {}",
                hex::encode(&credential_id)
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?;

    // Validate counter to prevent replay attacks (database i32, WebAuthn u32)
    let new_counter = auth_result.counter();
    if new_counter <= stored_credential.counter as u32 {
        //
        tracing::error!(
            "Counter replay attack detected for user '{}': stored={}, provided={}",
            req.username,
            stored_credential.counter,
            new_counter
        );
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Authentication failed".to_string(),
            }),
        ));
    }

    // Update credential with new counter value. Database i32, WebAuthn u32; Safe casts
    // since counter will never exceed i32::MAX in practice (will take 5000 years at 1000
    // auths per day for a single user)
    stored_credential.counter = new_counter as i32;
    state
        .repository()
        .update_credential(stored_credential.clone())
        .await
        .map_err(|e| {
            //
            tracing::error!("Failed to update credential counter: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?;

    // Get user for session creation
    let user = state
        .repository()
        .get_user_by_id(stored_credential.user_id)
        .await
        .map_err(|e| {
            //
            tracing::error!("Database error fetching user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            //
            tracing::error!("User not found for credential");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?;

    // Create session token
    let session_token = session::create_session(&mut conn, user.id, user.username.clone())
        .await
        .map_err(|status| {
            //
            tracing::error!("Failed to create session for user: {}", user.username);
            (
                status,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                }),
            )
        })?;

    tracing::info!("User '{}' authenticated successfully", req.username);

    Ok(Json(AuthFinishResponse {
        session_token,
        success: true,
    }))
}
