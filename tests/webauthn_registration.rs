//! Integration tests for WebAuthn registration endpoints.
//!
//! Tests the full registration flow including:
//! - Challenge generation and storage
//! - Challenge expiration (TTL)
//! - Credential verification API contract
//! - Error handling
//!
//! ## Testing Limitations
//!
//! These tests validate the API layer but do NOT test actual WebAuthn
//! credential verification or counter validation. Full end-to-end testing
//! requires browser automation (e.g., Playwright) to generate real
//! authenticator responses.
//!
//! **TODO (Future Work):**
//! - Add e2e tests with Playwright for full credential flow
//! - Test counter validation with real authenticator responses
//! - Test replay attack prevention
//!
//! **Reference:** The cr8s project demonstrates e2e WebAuthn testing with Playwright.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_quickstart::create_router;
use once_cell::sync::Lazy;
use redis::Client;
use serde_json::json;
use std::env;
use tokio::runtime::Runtime;
use tower::ServiceExt;

mod common;

static TEST_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("failed to create Tokio runtime"));

// Test helper to run a test on the TEST_RUNTIME
pub fn run_async<F>(fut: F)
where
    F: std::future::Future<Output = ()>,
{
    TEST_RUNTIME.block_on(fut)
}

/// Cleanup Redis keys after test (async implementation).
async fn cleanup_redis(username: &str) {
    // ---
    let redis_url = env::var("REDIS_URL").unwrap();
    let client = Client::open(redis_url).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();

    let key = format!("webauthn:reg:{username}");
    let _: () = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

// ============================================================================
// Registration Start Tests
// ============================================================================

#[test]
fn test_register_start_creates_challenge() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let app = create_router().expect("Failed to create router");
        let username = "test_user_start@example.com";

        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/start")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify challenge response structure
        assert!(json.get("challenge").is_some());
        let challenge = json.get("challenge").unwrap();
        assert!(challenge.get("publicKey").is_some());

        cleanup_redis(username).await;
    })
}

#[test]
fn test_register_start_creates_user_if_not_exists() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let app = create_router().expect("Failed to create router");
        let username = "new_user@example.com";

        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/start")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Calling again should succeed (user already exists)
        let app = create_router().expect("Failed to create router");
        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/start")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        cleanup_redis(username).await;
    })
}

#[test]
fn test_register_start_stores_challenge_in_redis() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let app = create_router().expect("Failed to create router");
        let username = "redis_test_user@example.com";

        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/start")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify challenge is in Redis
        let redis_url = env::var("REDIS_URL").unwrap();
        let client = Client::open(redis_url).unwrap();
        let mut conn = client.get_multiplexed_async_connection().await.unwrap();

        let key = format!("webauthn:reg:{username}");
        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();

        assert!(exists, "Challenge should be stored in Redis");

        cleanup_redis(username).await;
    })
}

// ============================================================================
// Registration Finish Tests
// ============================================================================

#[ignore = "Requires injectable WebAuthn verifier; see GH-33"]
#[test]
fn test_register_finish_fails_without_challenge() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let app = create_router().expect("Failed to create router");
        let username = "no_challenge_user@example.com";

        // Try to finish registration without starting it
        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/finish")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username,
                    "credential": {
                        "id": "fake_credential_id",
                        "rawId": "fake_raw_id",
                        "type": "public-key",
                        "response": {
                            "attestationObject": "fake_attestation",
                            "clientDataJSON": "fake_client_data",
                            "signature": "fake_signature",
                            "userHandle": null
                        },
                        "type": "public-key"
                    }
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should fail because challenge doesn't exist
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json
            .get("error")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("not found or expired"));
    })
}

#[ignore = "Requires injectable WebAuthn verifier; see GH-33"]
#[test]
fn test_register_finish_challenge_is_single_use() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let username = "single_use_user@example.com";

        // Start registration to create challenge
        let app = create_router().expect("Failed to create router");
        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/start")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Try to finish with invalid credential (will fail but consume challenge)
        let app = create_router().expect("Failed to create router");
        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/finish")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username,
                    "credential": {
                        "id": "fake_credential_id",
                        "rawId": "fake_raw_id",
                        "type": "public-key",
                        "response": {
                            "attestationObject": "fake_attestation",
                            "clientDataJSON": "fake_client_data",
                            "signature": "fake_signature",
                            "userHandle": null
                        },
                        "type": "public-key"
                    }
                })
                .to_string(),
            ))
            .unwrap();

        let _response = app.oneshot(request).await.unwrap();

        // Verify challenge is deleted from Redis
        let redis_url = env::var("REDIS_URL").unwrap();
        let client = Client::open(redis_url).unwrap();
        let mut conn = client.get_multiplexed_async_connection().await.unwrap();

        let key = format!("webauthn:reg:{username}");
        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();

        assert!(!exists, "Challenge should be deleted after use");

        cleanup_redis(username).await;
    })
}

// ============================================================================
// Challenge Expiration Tests
// ============================================================================

#[test]
fn test_challenge_has_ttl_in_redis() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let app = create_router().expect("Failed to create router");
        let username = "ttl_test_user@example.com";

        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/start")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check TTL in Redis
        let redis_url = env::var("REDIS_URL").unwrap();
        let client = Client::open(redis_url).unwrap();
        let mut conn = client.get_multiplexed_async_connection().await.unwrap();

        let key = format!("webauthn:reg:{username}");
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();

        // TTL should be set (default 300 seconds = 5 minutes)
        assert!(ttl > 0, "TTL should be positive");
        assert!(ttl <= 300, "TTL should be <= 300 seconds");

        cleanup_redis(username).await;
    })
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_register_start_invalid_json() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let app = create_router().expect("Failed to create router");

        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/start")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 4xx for invalid JSON
        assert!(response.status().is_client_error());
    })
}

#[test]
fn test_register_finish_invalid_json() {
    // ---
    run_async(async {
        // ---
        common::setup_test_env().await;

        let app = create_router().expect("Failed to create router");

        let request = Request::builder()
            .method("POST")
            .uri("/webauthn/register/finish")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 4xx for invalid JSON
        assert!(response.status().is_client_error());
    })
}
