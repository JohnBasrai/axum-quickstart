use axum_quickstart::create_postgres_repository;
use axum_quickstart::domain::Credential; // {Credential, Repository, User};
use sqlx::PgPool;
use uuid::Uuid;

// Helper to get test database URL from environment or use default
fn get_test_database_url() -> String {
    // ---
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/axum_quickstart_test".to_string()
    })
}

// Helper to setup test database and run migrations
async fn setup_test_db() -> PgPool {
    // ---
    let database_url = get_test_database_url();

    // Connect to database
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

// Helper to clean up test data after each test
async fn cleanup_test_db(pool: &PgPool) {
    // ---
    sqlx::query("TRUNCATE TABLE credentials, users CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean up test database");
}

#[tokio::test]
async fn test_create_and_get_user() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Create a user
    let username = "alice";
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

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

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

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_username_must_be_unique() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    let username = "bob";

    // Create first user
    repo.create_user(username)
        .await
        .expect("First user should succeed");

    // Try to create second user with same username
    let result = repo.create_user(username).await;

    assert!(result.is_err(), "Duplicate username should fail");

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_save_and_get_credential() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Create a user first
    let user = repo
        .create_user("charlie")
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

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_get_credentials_by_user() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Create a user
    let user = repo
        .create_user("dave")
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

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_update_credential_counter() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Create user and credential
    let user = repo
        .create_user("eve")
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

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_delete_credential() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Create user and credential
    let user = repo
        .create_user("frank")
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

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_foreign_key_constraint() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Create user
    let user = repo
        .create_user("grace")
        .await
        .expect("Failed to create user");
    let credential_id = vec![7, 7, 7];
    let credential = Credential::new(credential_id, user.id, vec![70, 70, 70], 0);

    repo.save_credential(credential)
        .await
        .expect("Failed to save credential");

    // Delete user (should cascade delete credentials)
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .expect("Failed to delete user");

    // Verify credential was also deleted (CASCADE)
    let creds = repo
        .get_credentials_by_user(user.id)
        .await
        .expect("Failed to get credentials");

    assert_eq!(creds.len(), 0);

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_credential_without_user_fails() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Try to create credential with nonexistent user
    let nonexistent_user_id = Uuid::new_v4();
    let credential = Credential::new(vec![8, 8, 8], nonexistent_user_id, vec![80, 80, 80], 0);

    let result = repo.save_credential(credential).await;

    assert!(result.is_err(), "Credential without valid user should fail");

    cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_multiple_credentials_per_user() {
    // ---
    let pool = setup_test_db().await;
    let repo = create_postgres_repository(pool.clone());

    // Create user
    let user = repo
        .create_user("henry")
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

    cleanup_test_db(&pool).await;
}
