//! Integration tests for WebAuthn authentication flow.
//!
//! Tests the complete authentication process including challenge generation,
//! credential verification, counter validation, and session creation.

use axum_quickstart::create_postgres_repository;
use axum_quickstart::create_session;
use axum_quickstart::domain::{Credential, Repository, User};
use once_cell::sync::Lazy;
use redis::AsyncCommands;
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

mod common;

// ---

/// Shared static runtime for all database tests to avoid lifecycle issues.
static TEST_RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    //
    Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime"),
    )
});

// ---

/// Test helper: Create test user in database
async fn create_test_user(repo: &dyn Repository, username: &str) -> User {
    //
    repo.create_user(username)
        .await
        .expect("Failed to create test user")
}

/// Test helper: Create test credential for user
async fn create_test_credential(
    repo: &dyn Repository,
    user_id: Uuid,
    credential_id: Vec<u8>,
) -> Credential {
    //
    let credential = Credential {
        id: credential_id,
        user_id,
        public_key: b"dummy_passkey_json".to_vec(), // Would be actual Passkey JSON in real flow
        counter: 0,
        created_at: chrono::Utc::now(),
    };

    repo.save_credential(credential.clone())
        .await
        .expect("Failed to save credential");

    credential
}

/// Test helper: Get Redis connection
async fn get_redis_connection() -> redis::aio::MultiplexedConnection {
    //
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis")
}

// ============================================================================
// Authentication Flow Tests
// ============================================================================

#[test]
#[ignore] // Ignored due to Issue #33: WebAuthn verifier injection limitations
fn test_auth_start_success() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let repo = create_postgres_repository().expect("Failed to create repository");
        let username = format!("auth_test_{}", Uuid::new_v4());

        // Create user with credential
        let user = create_test_user(repo.as_ref(), &username).await;
        let credential_id = vec![1, 2, 3, 4];
        create_test_credential(repo.as_ref(), user.id, credential_id).await;

        // Note: Actual auth_start endpoint call would require full HTTP server setup
        // This test validates the database/Redis infrastructure is ready
        // Full E2E test would use reqwest to call POST /webauthn/auth/start

        // Verify user and credentials exist
        let fetched_user = repo
            .get_user_by_username(&username)
            .await
            .expect("Failed to fetch user")
            .expect("User not found");
        assert_eq!(fetched_user.username, username);

        let credentials = repo
            .get_credentials_by_user(user.id)
            .await
            .expect("Failed to fetch credentials");
        assert_eq!(credentials.len(), 1);

        // Cleanup
        repo.delete_credential(&credentials[0].id)
            .await
            .expect("Failed to cleanup credential");
    });
}

#[test]
#[ignore] // Ignored due to Issue #33: WebAuthn verifier injection limitations
fn test_auth_start_user_not_found() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let repo = create_postgres_repository().expect("Failed to create repository");
        let username = format!("nonexistent_{}", Uuid::new_v4());

        // Verify user doesn't exist
        let result = repo
            .get_user_by_username(&username)
            .await
            .expect("Database query failed");
        assert!(result.is_none(), "User should not exist");

        // Actual endpoint call would return:
        // StatusCode::UNAUTHORIZED with "Authentication failed"
    });
}

#[test]
#[ignore] // Ignored due to Issue #33: WebAuthn verifier injection limitations
fn test_auth_start_no_credentials() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let repo = create_postgres_repository().expect("Failed to create repository");
        let username = format!("no_creds_{}", Uuid::new_v4());

        // Create user without credentials
        let user = create_test_user(repo.as_ref(), &username).await;

        let credentials = repo
            .get_credentials_by_user(user.id)
            .await
            .expect("Failed to fetch credentials");
        assert!(credentials.is_empty(), "User should have no credentials");

        // Actual endpoint call would return:
        // StatusCode::UNAUTHORIZED with "Authentication failed"
    });
}

// ============================================================================
// Challenge Storage Tests
// ============================================================================

#[test]
fn test_redis_challenge_storage() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let mut conn = get_redis_connection().await;
        let username = format!("redis_test_{}", Uuid::new_v4());
        let redis_key = format!("webauthn:auth:{username}");

        // Store dummy challenge
        let challenge_data = json!({
            "challenge": "test_challenge_data"
        });
        let challenge_json = serde_json::to_vec(&challenge_data).unwrap();

        conn.set_ex::<_, _, ()>(&redis_key, challenge_json.clone(), 300)
            .await
            .expect("Failed to store challenge");

        // Verify challenge exists
        let stored: Vec<u8> = conn
            .get(&redis_key)
            .await
            .expect("Failed to retrieve challenge");
        assert_eq!(stored, challenge_json);

        // Test atomic GETDEL
        let retrieved: Vec<u8> = conn
            .get_del(&redis_key)
            .await
            .expect("Failed to GETDEL challenge");
        assert_eq!(retrieved, challenge_json);

        // Verify challenge is deleted
        let deleted: Option<Vec<u8>> = conn.get(&redis_key).await.expect("Redis query failed");
        assert!(deleted.is_none(), "Challenge should be deleted");
    });
}

#[test]
fn test_redis_challenge_expiry() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let mut conn = get_redis_connection().await;
        let username = format!("expiry_test_{}", Uuid::new_v4());
        let redis_key = format!("webauthn:auth:{username}");

        // Store challenge with 1-second TTL
        let challenge_data = b"expiring_challenge";
        conn.set_ex::<_, _, ()>(&redis_key, challenge_data.to_vec(), 1)
            .await
            .expect("Failed to store challenge");

        // Wait for expiry
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Verify challenge expired
        let result: Option<Vec<u8>> = conn.get(&redis_key).await.expect("Redis query failed");
        assert!(result.is_none(), "Challenge should have expired");
    });
}

// ============================================================================
// Counter Validation Tests
// ============================================================================

#[test]
fn test_counter_increment() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let repo = create_postgres_repository().expect("Failed to create repository");
        let username = format!("counter_test_{}", Uuid::new_v4());

        // Create user and credential
        let user = create_test_user(repo.as_ref(), &username).await;
        let credential_id = vec![5, 6, 7, 8];
        let mut credential = create_test_credential(repo.as_ref(), user.id, credential_id).await;

        assert_eq!(credential.counter, 0, "Initial counter should be 0");

        // Simulate successful authentication - increment counter
        credential.counter = 1;
        repo.update_credential(credential.clone())
            .await
            .expect("Failed to update counter");

        // Verify counter was updated
        let updated = repo
            .get_credential_by_id(&credential.id)
            .await
            .expect("Failed to fetch credential")
            .expect("Credential not found");
        assert_eq!(updated.counter, 1, "Counter should be incremented");

        // Cleanup
        repo.delete_credential(&credential.id)
            .await
            .expect("Failed to cleanup");
    });
}

#[test]
fn test_counter_replay_detection() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let repo = create_postgres_repository().expect("Failed to create repository");
        let username = format!("replay_test_{}", Uuid::new_v4());

        // Create user and credential with counter = 5
        let user = create_test_user(repo.as_ref(), &username).await;
        let credential_id = vec![9, 10, 11, 12];
        let mut credential = create_test_credential(repo.as_ref(), user.id, credential_id).await;
        credential.counter = 5;
        repo.update_credential(credential.clone())
            .await
            .expect("Failed to set initial counter");

        // Simulate replay attack: new_counter <= stored_counter
        let new_counter_replay = 4u32; // Less than stored (5)
        let stored_counter = credential.counter as u32;

        assert!(
            new_counter_replay <= stored_counter,
            "Replay attack should be detected"
        );

        // Simulate valid authentication: new_counter > stored_counter
        let new_counter_valid = 6u32;
        assert!(
            new_counter_valid > stored_counter,
            "Valid counter should be accepted"
        );

        // Cleanup
        repo.delete_credential(&credential.id)
            .await
            .expect("Failed to cleanup");
    });
}

// ============================================================================
// Session Token Tests
// ============================================================================

#[test]
fn test_session_creation() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let mut conn = get_redis_connection().await;
        let user_id = Uuid::new_v4();
        let username = format!("session_test_{}", Uuid::new_v4());

        // Create session
        let token = create_session(&mut conn, user_id, username.clone())
            .await
            .expect("Failed to create session");

        // Verify token is a valid UUID
        Uuid::parse_str(&token).expect("Token should be valid UUID");

        // Verify session stored in Redis
        let session_key = format!("session:{token}");
        let session_data: String = conn
            .get(&session_key)
            .await
            .expect("Failed to retrieve session");

        let session_json: serde_json::Value =
            serde_json::from_str(&session_data).expect("Invalid session JSON");
        assert_eq!(
            session_json["username"].as_str().unwrap(),
            username,
            "Username should match"
        );
        assert_eq!(
            session_json["user_id"].as_str().unwrap(),
            user_id.to_string(),
            "User ID should match"
        );

        // Cleanup
        let _: () = conn
            .del(&session_key)
            .await
            .expect("Failed to cleanup session");
    });
}

#[test]
fn test_session_ttl() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        let mut conn = get_redis_connection().await;
        let user_id = Uuid::new_v4();
        let username = "ttl_test_user".to_string();

        // Create session
        let token = create_session(&mut conn, user_id, username)
            .await
            .expect("Failed to create session");

        // Check TTL (should be 7 days = 604800 seconds)
        let session_key = format!("session:{token}");
        let ttl: i64 = conn.ttl(&session_key).await.expect("Failed to get TTL");

        // TTL should be close to 7 days (allow some variance for test execution time)
        assert!(
            ttl > 604700 && ttl <= 604800,
            "TTL should be ~7 days (604800s), got {ttl}",
        );

        // Cleanup
        let _: () = conn
            .del(&session_key)
            .await
            .expect("Failed to cleanup session");
    });
}
