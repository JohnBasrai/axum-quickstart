//! WebAuthn registration handlers.
//!
//! Implements the two-phase passkey registration flow:
//! 1. `register_start` - Generate challenge and return credential creation options
//! 2. `register_finish` - Verify credential and store in database

use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RegistrationStartRequest {
    // ---
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct RegistrationStartResponse {
    // ---
    pub challenge: CreationChallengeResponse,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationFinishRequest {
    // ---
    pub username: String,
    pub credential: RegisterPublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub struct RegistrationFinishResponse {
    // ---
    pub success: bool,
    pub credential_id: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    // ---
    pub error: String,
}

// ============================================================================
// Registration Start Handler
// ============================================================================

/// POST /webauthn/register/start
///
/// Initiates passkey registration by generating a WebAuthn challenge.
/// The challenge is stored in Redis with a TTL and must be used in the
/// finish endpoint before expiration.
///
/// # Request Body
/// ```json
/// { "username": "user@example.com" }
/// ```
///
/// # Response
/// Returns WebAuthn credential creation options containing the challenge.
/// The client passes these options to `navigator.credentials.create()`.
pub async fn register_start(
    State(state): State<AppState>,
    Json(req): Json<RegistrationStartRequest>,
) -> Result<Json<RegistrationStartResponse>, (StatusCode, Json<ErrorResponse>)> {
    // ---

    // Create or get user from database
    let user = state
        .repository()
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
        })?;

    let user = match user {
        Some(u) => u,
        None => {
            // Create new user
            state
                .repository()
                .create_user(&req.username)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to create user: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Failed to create user".to_string(),
                        }),
                    )
                })?
        }
    };

    // Generate WebAuthn challenge
    let (challenge_response, registration_state) = state
        .webauthn()
        .start_passkey_registration(user.id, &req.username, &req.username, None)
        .map_err(|e| {
            tracing::error!("Failed to start registration: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to generate challenge".to_string(),
                }),
            )
        })?;

    // Store registration state in Redis with TTL (using bincode)
    let state_key = format!("webauthn:reg:{}", req.username);
    let state_bytes = serde_json::to_vec(&registration_state).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("failed to serialize webauthn registration state: {e}"),
            }),
        )
    })?;

    let mut conn = state.get_conn().await.map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: "Redis connection failed".to_string(),
            }),
        )
    })?;

    let ttl_secs = state.challenge_ttl().as_secs();
    let _: () = conn
        .set_ex(&state_key, state_bytes, ttl_secs)
        .await
        .map_err(|e| {
            tracing::error!("Failed to store challenge in Redis: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to store challenge".to_string(),
                }),
            )
        })?;

    tracing::info!("Registration started for user: {}", req.username);

    Ok(Json(RegistrationStartResponse {
        challenge: challenge_response,
    }))
}

// ============================================================================
// Registration Finish Handler
// ============================================================================

/// POST /webauthn/register/finish
///
/// Completes passkey registration by verifying the credential from the
/// authenticator and storing it in the database.
///
/// # Request Body
/// Contains the username and the credential returned by the authenticator
/// via `navigator.credentials.create()`.
///
/// # Response
/// Returns success status and the credential ID if verification succeeds.
pub async fn register_finish(
    State(state): State<AppState>,
    Json(req): Json<RegistrationFinishRequest>,
) -> Result<Json<RegistrationFinishResponse>, (StatusCode, Json<ErrorResponse>)> {
    // ---

    // Retrieve registration state from Redis
    let state_key = format!("webauthn:reg:{}", req.username);
    let mut conn = state.get_conn().await.map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: "Redis connection failed".to_string(),
            }),
        )
    })?;

    // A challenge must be consumed, not fetched then deleted later, i.e. this must
    // be atomic
    let state_bytes: Vec<u8> = conn.get_del(&state_key).await.map_err(|e| {
        tracing::warn!("Challenge not found or expired for user: {}", req.username);
        tracing::debug!("Redis error: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Challenge not found or expired".to_string(),
            }),
        )
    })?;

    let registration_state: PasskeyRegistration =
        serde_json::from_slice(&state_bytes).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("failed to deserialize webauthn registration state: {e}"),
                }),
            )
        })?;

    // Verify the credential
    let passkey = state
        .webauthn()
        .finish_passkey_registration(&req.credential, &registration_state)
        .map_err(|e| {
            tracing::error!("Credential verification failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Credential verification failed".to_string(),
                }),
            )
        })?;

    // Get user from database
    let user = state
        .repository()
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                }),
            )
        })?;

    // Store credential in database
    // Note: Passkey is serialized as the public_key, counter is extracted separately
    let cred_id = passkey.cred_id().to_vec();
    let passkey_bytes = serde_json::to_vec(&passkey).map_err(|e| {
        tracing::error!("Failed to serialize passkey: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Serialization error".to_string(),
            }),
        )
    })?;

    let credential = crate::domain::Credential::new(
        cred_id.clone(),
        user.id,
        passkey_bytes,
        0, // Initial counter value for new credentials
    );

    state
        .repository()
        .save_credential(credential)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save credential: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to save credential".to_string(),
                }),
            )
        })?;

    let cred_id_hex = hex::encode(&cred_id);
    tracing::info!(
        "Registration completed for user: {} (credential: {})",
        req.username,
        cred_id_hex
    );

    Ok(Json(RegistrationFinishResponse {
        success: true,
        credential_id: cred_id_hex,
    }))
}
