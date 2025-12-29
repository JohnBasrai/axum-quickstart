//! WebAuthn credential management handlers.
//!
//! Implements credential management operations (Phase 4):
//! 1. `list_credentials` - List all passkeys for authenticated user
//! 2. `delete_credential` - Remove a specific passkey

use crate::app_state::AppState;
use crate::session;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::Engine;
use serde::Serialize;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Response containing a user's registered credentials.
#[derive(Debug, Serialize)]
pub struct ListCredentialsResponse {
    // ---
    pub credentials: Vec<CredentialInfo>,
}

// ---

/// Information about a registered credential (passkey).
///
/// This is a sanitized view of credential data suitable for display to users.
/// Private keys and other sensitive cryptographic material are never exposed.
#[derive(Debug, Serialize)]
pub struct CredentialInfo {
    // ---
    /// Base64-encoded credential ID
    pub id: String,
    /// When this credential was registered
    pub created_at: String,
}

// ---

/// Response for successful credential deletion.
#[derive(Debug, Serialize)]
pub struct DeleteCredentialResponse {
    // ---
    pub success: bool,
    pub message: String,
}

// ---

/// Error response for credential management operations.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    // ---
    pub error: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extracts and validates the session token from Authorization header.
///
/// Expects header format: "Authorization: Bearer <token>"
///
/// # Security
///
/// - Validates token exists in Redis
/// - Returns authenticated user's ID for authorization checks
///
/// # Errors
///
/// Returns UNAUTHORIZED if:
/// - Authorization header is missing
/// - Header format is invalid (not "Bearer <token>")
/// - Token is invalid or expired
async fn extract_session(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<session::SessionInfo, (StatusCode, Json<ErrorResponse>)> {
    // ---
    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| {
            // ---
            tracing::debug!("Missing Authorization header");
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing Authorization header".to_string(),
                }),
            )
        })?
        .to_str()
        .map_err(|_| {
            // ---
            tracing::debug!("Invalid Authorization header format");
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid Authorization header".to_string(),
                }),
            )
        })?;

    // Parse Bearer token
    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        // ---
        tracing::debug!("Authorization header missing Bearer prefix");
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid Authorization header format".to_string(),
            }),
        )
    })?;

    // Validate session with Redis
    let mut redis_conn = state.get_conn().await.map_err(|status| {
        // ---
        (
            status,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
            }),
        )
    })?;

    session::validate_session(&mut redis_conn, token)
        .await
        .map_err(|status| {
            // ---
            (
                status,
                Json(ErrorResponse {
                    error: "Invalid or expired session".to_string(),
                }),
            )
        })
}

// ============================================================================
// List Credentials Handler
// ============================================================================

/// GET /webauthn/credentials
///
/// Lists all WebAuthn credentials (passkeys) registered by the authenticated user.
///
/// This is part of the credential management phase (Phase 4), allowing users to view
/// which devices/authenticators they have registered for their account.
///
/// # Security
///
/// - Requires valid session token in Authorization header (Bearer token)
/// - Only returns credentials owned by the authenticated user
/// - Credential private keys are never exposed (only IDs and metadata)
///
/// # Request Headers
/// ```text
/// Authorization: Bearer <session_token>
/// ```
///
/// # Response
/// Returns a list of credential IDs and creation timestamps.
///
/// # Errors
///
/// Returns an error if:
/// - Session token is missing or invalid (401 Unauthorized)
/// - Database query fails (500 Internal Server Error)
pub async fn list_credentials(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListCredentialsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // ---
    // Validate session and extract user_id
    let session_info = extract_session(&headers, &state).await?;

    tracing::info!(
        "Listing credentials for user: {} ({})",
        session_info.username,
        session_info.user_id
    );

    // Fetch user's credentials from database
    let credentials = state
        .repository()
        .get_credentials_by_user(session_info.user_id)
        .await
        .map_err(|e| {
            // ---
            tracing::error!(
                "Failed to fetch credentials for user {}: {}",
                session_info.user_id,
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch credentials".to_string(),
                }),
            )
        })?;

    // Convert to response format (sanitized view)
    let credential_list: Vec<CredentialInfo> = credentials
        .into_iter()
        .map(|cred| {
            // ---
            CredentialInfo {
                id: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&cred.id),
                created_at: cred.created_at.to_rfc3339(),
            }
        })
        .collect();

    tracing::info!(
        "Found {} credentials for user: {}",
        credential_list.len(),
        session_info.username
    );

    Ok(Json(ListCredentialsResponse {
        credentials: credential_list,
    }))
}

// ============================================================================
// Delete Credential Handler
// ============================================================================

/// DELETE /webauthn/credentials/:id
///
/// Deletes a specific WebAuthn credential (passkey) for the authenticated user.
///
/// This allows users to revoke access for lost devices or remove old authenticators.
///
/// # Security
///
/// - Requires valid session token in Authorization header (Bearer token)
/// - Verifies credential belongs to authenticated user before deletion
/// - Prevents unauthorized deletion of other users' credentials
///
/// # Request Headers
/// ```text
/// Authorization: Bearer <session_token>
/// ```
///
/// # Path Parameters
/// - `id` - Base64-encoded credential ID to delete
///
/// # Errors
///
/// Returns an error if:
/// - Session token is missing or invalid (401 Unauthorized)
/// - Credential ID is invalid base64 (400 Bad Request)
/// - Credential doesn't exist (404 Not Found)
/// - Credential belongs to different user (403 Forbidden)
/// - Database deletion fails (500 Internal Server Error)
pub async fn delete_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(credential_id_base64): Path<String>,
) -> Result<Json<DeleteCredentialResponse>, (StatusCode, Json<ErrorResponse>)> {
    // ---
    // Validate session and extract user_id
    let session_info = extract_session(&headers, &state).await?;

    tracing::info!(
        "Deleting credential {} for user: {} ({})",
        credential_id_base64,
        session_info.username,
        session_info.user_id
    );

    // Decode credential ID from base64
    let credential_id = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&credential_id_base64)
        .map_err(|e| {
            // ---
            tracing::warn!("Invalid base64 credential ID: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid credential ID format".to_string(),
                }),
            )
        })?;

    // Verify credential exists and belongs to this user
    let credential = state
        .repository()
        .get_credential_by_id(&credential_id)
        .await
        .map_err(|e| {
            // ---
            tracing::error!("Failed to query credential: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to query credential".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            // ---
            tracing::warn!("Credential not found: {}", credential_id_base64);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Credential not found".to_string(),
                }),
            )
        })?;

    // Verify ownership (prevent deletion of other users' credentials)
    if credential.user_id != session_info.user_id {
        // ---
        tracing::warn!(
            "User {} attempted to delete credential belonging to user {}",
            session_info.user_id,
            credential.user_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Cannot delete credential belonging to another user".to_string(),
            }),
        ));
    }

    // Delete credential from database
    state
        .repository()
        .delete_credential(&credential_id)
        .await
        .map_err(|e| {
            // ---
            tracing::error!("Failed to delete credential: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete credential".to_string(),
                }),
            )
        })?;

    tracing::info!(
        "Successfully deleted credential {} for user {}",
        credential_id_base64,
        session_info.username
    );

    Ok(Json(DeleteCredentialResponse {
        success: true,
        message: "Credential deleted successfully".to_string(),
    }))
}
