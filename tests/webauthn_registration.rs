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
use axum_quickstart::{create_router, domain::init_database_with_retry_from_env};
use redis::Client;
use serde_json::json;
use serial_test::serial;
use std::env;
use tower::ServiceExt;

// ============================================================================
// Test Setup
// ============================================================================

/// Initialize test environment (database, Redis, env vars).
async fn setup_test_env() {
    // ---
    // Set required environment variables for testing
    env::set_var(
        "DATABASE_URL",
        "postgres://postgres:postgres@localhost:5432/axum_quickstart",
    );
    env::set_var("AXUM_REDIS_URL", "redis://127.0.0.1:6379");
    env::set_var("AXUM_WEBAUTHN_RP_ID", "localhost");
    env::set_var("AXUM_WEBAUTHN_ORIGIN", "http://localhost:8080");
    env::set_var("AXUM_WEBAUTHN_RP_NAME", "Test App");
    env::set_var("AXUM_METRICS_TYPE", "noop");

    // Initialize database (runs migrations)
    init_database_with_retry_from_env()
        .await
        .expect("Failed to initialize database");
}

/// Cleanup Redis keys after test.
async fn cleanup_redis(username: &str) {
    // ---
    let redis_url = env::var("AXUM_REDIS_URL").unwrap();
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

#[tokio::test]
#[serial]
async fn test_register_start_creates_challenge() {
    // ---
    setup_test_env().await;

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
}

#[tokio::test]
#[serial]
async fn test_register_start_creates_user_if_not_exists() {
    // ---
    setup_test_env().await;

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
}

#[tokio::test]
#[serial]
async fn test_register_start_stores_challenge_in_redis() {
    // ---
    setup_test_env().await;

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
    let redis_url = env::var("AXUM_REDIS_URL").unwrap();
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
}

// ============================================================================
// Registration Finish Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_register_finish_fails_without_challenge() {
    // ---
    setup_test_env().await;

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
                    "response": {
                        "attestationObject": "fake_attestation",
                        "clientDataJSON": "fake_client_data"
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
}

#[tokio::test]
#[serial]
async fn test_register_finish_challenge_is_single_use() {
    // ---
    setup_test_env().await;

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
                    "response": {
                        "attestationObject": "fake_attestation",
                        "clientDataJSON": "fake_client_data"
                    },
                    "type": "public-key"
                }
            })
            .to_string(),
        ))
        .unwrap();

    let _response = app.oneshot(request).await.unwrap();

    // Verify challenge is deleted from Redis
    let redis_url = env::var("AXUM_REDIS_URL").unwrap();
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
}

// ============================================================================
// Challenge Expiration Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_challenge_has_ttl_in_redis() {
    // ---
    setup_test_env().await;

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
    let redis_url = env::var("AXUM_REDIS_URL").unwrap();
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
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_register_start_invalid_json() {
    // ---
    setup_test_env().await;

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
}

#[tokio::test]
#[serial]
async fn test_register_finish_invalid_json() {
    // ---
    setup_test_env().await;

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
}
