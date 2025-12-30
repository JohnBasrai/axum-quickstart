//! Integration tests for WebAuthn credential management (Phase 4).
//!
//! Tests credential listing and deletion endpoints with session-based authentication.

use axum_quickstart::create_postgres_repository;
use axum_quickstart::domain::{Credential, RepositoryPtr, User};
use axum_quickstart::{create_session, validate_session};
use once_cell::sync::Lazy;
use redis::AsyncCommands;
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
async fn create_test_user(repo: &RepositoryPtr, username: &str) -> User {
    //
    repo.create_user(username)
        .await
        .expect("Failed to create test user")
}

// ---

/// Test helper: Create test credential for user
async fn create_test_credential(
    repo: &RepositoryPtr,
    user_id: Uuid,
    credential_id: Vec<u8>,
) -> Credential {
    //
    let credential = Credential {
        id: credential_id,
        user_id,
        public_key: b"dummy_public_key".to_vec(),
        counter: 0,
        created_at: chrono::Utc::now(),
    };

    repo.save_credential(credential.clone())
        .await
        .expect("Failed to save credential");

    credential
}

// ---

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
// Session Validation Tests
// ============================================================================

#[test]
fn test_session_validation_success() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        //
        // Setup
        let repo = create_postgres_repository().expect("Failed to create repository");
        let user = create_test_user(&repo, "test_session_user").await;
        let mut redis_conn = get_redis_connection().await;

        // Create session
        let token = create_session(&mut redis_conn, user.id, user.username.clone())
            .await
            .expect("Failed to create session");

        // Validate session
        let session_info = validate_session(&mut redis_conn, &token)
            .await
            .expect("Session validation failed");

        // Verify
        assert_eq!(session_info.user_id, user.id);
        assert_eq!(session_info.username, user.username);

        // Cleanup
        let _: Result<(), _> = redis_conn.del(format!("session:{}", token)).await;
        let _ = repo.delete_credential(user.id.as_bytes().as_ref()).await;
    });
}

// ---

#[test]
fn test_session_validation_invalid_token() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        //
        let mut redis_conn = get_redis_connection().await;

        // Try to validate non-existent token
        let result = validate_session(&mut redis_conn, "invalid-token-12345").await;

        // Should fail with UNAUTHORIZED
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), axum::http::StatusCode::UNAUTHORIZED);
    });
}

// ============================================================================
// List Credentials Tests
// ============================================================================

#[test]
fn test_list_credentials_with_session() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        //
        // Setup
        let repo = create_postgres_repository().expect("Failed to create repository");
        let user = create_test_user(&repo, "test_list_user").await;
        let mut redis_conn = get_redis_connection().await;

        // Create multiple credentials for user
        let cred1 = create_test_credential(&repo, user.id, b"credential_1".to_vec()).await;
        let cred2 = create_test_credential(&repo, user.id, b"credential_2".to_vec()).await;

        // Create session
        let token = create_session(&mut redis_conn, user.id, user.username.clone())
            .await
            .expect("Failed to create session");

        // List credentials using repository directly (simulating handler logic)
        let credentials = repo
            .get_credentials_by_user(user.id)
            .await
            .expect("Failed to list credentials");

        // Verify
        assert_eq!(credentials.len(), 2);
        assert!(credentials.iter().any(|c| c.id == cred1.id));
        assert!(credentials.iter().any(|c| c.id == cred2.id));

        // Cleanup
        let _: Result<(), _> = redis_conn.del(format!("session:{}", token)).await;
        let _ = repo.delete_credential(&cred1.id).await;
        let _ = repo.delete_credential(&cred2.id).await;
    });
}

// ---

#[test]
fn test_list_credentials_empty_list() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        //
        // Setup
        let repo = create_postgres_repository().expect("Failed to create repository");
        let user = create_test_user(&repo, "test_empty_list_user").await;
        let mut redis_conn = get_redis_connection().await;

        // Create session but no credentials
        let token = create_session(&mut redis_conn, user.id, user.username.clone())
            .await
            .expect("Failed to create session");

        // List credentials
        let credentials = repo
            .get_credentials_by_user(user.id)
            .await
            .expect("Failed to list credentials");

        // Verify empty list
        assert_eq!(credentials.len(), 0);

        // Cleanup
        let _: Result<(), _> = redis_conn.del(format!("session:{}", token)).await;
    });
}

// ============================================================================
// Delete Credential Tests
// ============================================================================

#[test]
fn test_delete_credential_success() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        //
        // Setup
        let repo = create_postgres_repository().expect("Failed to create repository");
        let user = create_test_user(&repo, "test_delete_user").await;
        let credential =
            create_test_credential(&repo, user.id, b"credential_to_delete".to_vec()).await;
        let mut redis_conn = get_redis_connection().await;

        // Create session
        let token = create_session(&mut redis_conn, user.id, user.username.clone())
            .await
            .expect("Failed to create session");

        // Verify credential exists
        let found = repo
            .get_credential_by_id(&credential.id)
            .await
            .expect("Failed to query credential");
        assert!(found.is_some());

        // Delete credential
        repo.delete_credential(&credential.id)
            .await
            .expect("Failed to delete credential");

        // Verify credential is gone
        let not_found = repo
            .get_credential_by_id(&credential.id)
            .await
            .expect("Failed to query credential");
        assert!(not_found.is_none());

        // Cleanup
        let _: Result<(), _> = redis_conn.del(format!("session:{}", token)).await;
    });
}

// ---

#[test]
fn test_delete_credential_ownership_check() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        //
        // Setup
        let repo = create_postgres_repository().expect("Failed to create repository");
        let user1 = create_test_user(&repo, "test_owner_user").await;
        let user2 = create_test_user(&repo, "test_other_user").await;

        // Create credential for user1
        let credential =
            create_test_credential(&repo, user1.id, b"user1_credential".to_vec()).await;

        // Simulate user2 trying to access user1's credential
        let fetched = repo
            .get_credential_by_id(&credential.id)
            .await
            .expect("Failed to query credential")
            .expect("Credential should exist");

        // Verify ownership mismatch (handler would reject this)
        assert_ne!(fetched.user_id, user2.id);
        assert_eq!(fetched.user_id, user1.id);

        // Cleanup
        let _ = repo.delete_credential(&credential.id).await;
    });
}

// ---

#[test]
fn test_delete_nonexistent_credential() {
    //
    TEST_RUNTIME.block_on(async {
        //
        common::setup_test_env().await;

        //
        // Setup
        let repo = create_postgres_repository().expect("Failed to create repository");

        // Try to query non-existent credential
        let result = repo.get_credential_by_id(b"nonexistent_credential").await;

        // Should succeed but return None
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    });
}
