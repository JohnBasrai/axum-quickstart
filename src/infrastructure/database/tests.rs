use super::postgres_repository::*;
use crate::domain::Credential; // {Credential, Repository, User};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use uuid::Uuid;

// One runtime to rule them all...
/// Shared tokio runtime for all database tests.
///
/// We must initialize the database once and tests must share it.  Each test also must
/// share this single runtime instead of creating a new one per test.  This keeps the
/// database connection pool alive across all tests. Without it, each `#[tokio::test]`
/// would create its own runtime, and when that runtime drops at test completion, the pool
/// connections would be closed, causing subsequent tests to timeout waiting for new
/// connections.
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    // ---
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create TOKIO runtime")
});

// Initialize tracing once for all tests
static TRACING_INIT: std::sync::Once = std::sync::Once::new();

fn init_tracing() {
    // ---
    TRACING_INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_ansi(false)  // No colorization, makes logs easier to read.
            .with_test_writer()
            .init();
    });
}

async fn init() {
    // ---

    init_tracing();

    crate::domain::init_database_with_retry_from_env()
        .await
        .expect("database init failed");
}

async fn setup_repo() -> crate::domain::RepositoryPtr {
    // ---
    crate::domain::init_database_with_retry_from_env()
        .await
        .expect("database init failed");

    create_postgres_repository().expect("repository creation failed")
}

#[test]
fn test_create_and_get_user() {
    // ---
    RUNTIME.block_on(async {
        // --
        init().await;
        let repo = setup_repo().await;

        // Create a user
        let username = "Thorin Oakenshield";
        let user = repo
            .create_user(username)
            .await
            .expect("Failed to create user");

        assert_eq!(user.username, username);
        assert!(!user.id.is_nil());

        // Get user by username
        let found = repo
            .get_user_by_username(username)
            .await
            .expect("Failed to get user")
            .expect("User not found");

        assert_eq!(found.id, user.id);
        assert_eq!(found.username, username);

        // Get user by ID
        let found_by_id = repo
            .get_user_by_id(user.id)
            .await
            .expect("Failed to get user by ID")
            .expect("User not found by ID");

        assert_eq!(found_by_id.id, user.id);
        assert_eq!(found_by_id.username, username);
    });
}

#[test]
fn test_get_nonexistent_user() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        // Try to get user that doesn't exist
        let result = repo
            .get_user_by_username("nonexistent")
            .await
            .expect("Query should succeed");

        assert!(result.is_none());

        // Try to get by nonexistent ID
        let result = repo
            .get_user_by_id(Uuid::new_v4())
            .await
            .expect("Query should succeed");

        assert!(result.is_none());
    });
}

#[test]
fn test_username_must_be_unique() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        let username = "Fili";

        // Create first user
        repo.create_user(username)
            .await
            .expect("First user should succeed");

        // Try to create second user with same username
        let result = repo.create_user(username).await;

        assert!(result.is_err(), "Duplicate username should fail");
    });
}

#[test]
fn test_save_and_get_credential() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        // Create a user first
        let user = repo
            .create_user("Kili")
            .await
            .expect("Failed to create user");

        // Create a credential
        let credential_id = vec![1, 2, 3, 4, 5];
        let public_key = vec![10, 20, 30, 40, 50];
        let credential = Credential::new(credential_id.clone(), user.id, public_key.clone(), 0);

        // Save credential
        repo.save_credential(credential.clone())
            .await
            .expect("Failed to save credential");

        // Get credential by ID
        let found = repo
            .get_credential_by_id(&credential_id)
            .await
            .expect("Failed to get credential")
            .expect("Credential not found");

        assert_eq!(found.id, credential_id);
        assert_eq!(found.user_id, user.id);
        assert_eq!(found.public_key, public_key);
        assert_eq!(found.counter, 0);
    });
}

#[test]
fn test_get_credentials_by_user() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        // Create a user
        let user = repo
            .create_user("Balin")
            .await
            .expect("Failed to create user");

        // Initially no credentials
        let creds = repo
            .get_credentials_by_user(user.id)
            .await
            .expect("Failed to get credentials");
        assert_eq!(creds.len(), 0);

        // Add first credential
        let cred1 = Credential::new(vec![1, 1, 1], user.id, vec![10, 10, 10], 0);
        repo.save_credential(cred1)
            .await
            .expect("Failed to save credential");

        // Add second credential
        let cred2 = Credential::new(vec![2, 2, 2], user.id, vec![20, 20, 20], 0);
        repo.save_credential(cred2)
            .await
            .expect("Failed to save credential");

        // Get all credentials for user
        let creds = repo
            .get_credentials_by_user(user.id)
            .await
            .expect("Failed to get credentials");

        assert_eq!(creds.len(), 2);
    });
}

#[test]
fn test_update_credential_counter() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        // Create user and credential
        let user = repo
            .create_user("Dwalin")
            .await
            .expect("Failed to create user");
        let credential_id = vec![5, 5, 5];
        let public_key = vec![50, 50, 50];
        let mut credential = Credential::new(credential_id.clone(), user.id, public_key, 0);

        repo.save_credential(credential.clone())
            .await
            .expect("Failed to save credential");

        // Update counter (simulate authentication)
        credential.counter = 1;
        repo.update_credential(credential.clone())
            .await
            .expect("Failed to update credential");

        // Verify counter was updated
        let found = repo
            .get_credential_by_id(&credential_id)
            .await
            .expect("Failed to get credential")
            .expect("Credential not found");

        assert_eq!(found.counter, 1);

        // Update counter again
        credential.counter = 5;
        repo.update_credential(credential)
            .await
            .expect("Failed to update credential");

        let found = repo
            .get_credential_by_id(&credential_id)
            .await
            .expect("Failed to get credential")
            .expect("Credential not found");

        assert_eq!(found.counter, 5);
    });
}

#[test]
fn test_delete_credential() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        // Create user and credential
        let user = repo
            .create_user("Ori")
            .await
            .expect("Failed to create user");
        let credential_id = vec![6, 6, 6];
        let credential = Credential::new(credential_id.clone(), user.id, vec![60, 60, 60], 0);

        repo.save_credential(credential)
            .await
            .expect("Failed to save credential");

        // Verify credential exists
        let found = repo
            .get_credential_by_id(&credential_id)
            .await
            .expect("Failed to get credential");
        assert!(found.is_some());

        // Delete credential
        repo.delete_credential(&credential_id)
            .await
            .expect("Failed to delete credential");

        // Verify credential is gone
        let found = repo
            .get_credential_by_id(&credential_id)
            .await
            .expect("Failed to get credential");
        assert!(found.is_none());
    });
}

#[test]
fn test_z_credential_without_user_fails() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        // Try to create credential with nonexistent user
        let nonexistent_user_id = Uuid::new_v4();
        let credential = Credential::new(vec![8, 8, 8], nonexistent_user_id, vec![80, 80, 80], 0);

        let result = repo.save_credential(credential).await;

        assert!(result.is_err(), "Credential without valid user should fail");
    });
}

#[test]
fn test_multiple_credentials_per_user() {
    // ---
    RUNTIME.block_on(async {
        // ---
        init().await;
        let repo = setup_repo().await;

        // Create user
        let user = repo
            .create_user("Nori")
            .await
            .expect("Failed to create user");

        // Add multiple credentials (simulating multiple devices)
        let devices = vec![
            ("phone", vec![1, 0, 0]),
            ("laptop", vec![2, 0, 0]),
            ("yubikey", vec![3, 0, 0]),
        ];

        for (_, cred_id) in &devices {
            let credential = Credential::new(cred_id.clone(), user.id, vec![100, 100, 100], 0);
            repo.save_credential(credential)
                .await
                .expect("Failed to save credential");
        }

        // Get all credentials
        let creds = repo
            .get_credentials_by_user(user.id)
            .await
            .expect("Failed to get credentials");

        assert_eq!(creds.len(), 3);
    });
}
