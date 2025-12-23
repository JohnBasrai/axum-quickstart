use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::domain::{Credential, Repository, RepositoryPtr, User};

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct CredentialRow {
    id: Vec<u8>,
    user_id: Uuid,
    public_key: Vec<u8>,
    counter: i32,
    created_at: DateTime<Utc>,
}

/// Reads an environment variable and parses it into a value, falling back to a default.
/// Call this function somewhere early in main.rs before launching threads/tasks.
///
/// # Parameters
/// - `$name`: The name of the environment variable (string literal or expression)
/// - `$default`: A default value (any type that implements `Clone`)
///
#[macro_export]
macro_rules! get_env_with_default {
    ($ty:ty, $name:expr, $default:expr) => {{
        std::env::var($name)
            .map(|val| val.parse::<$ty>().unwrap_or($default))
            .unwrap_or($default)
    }};
}

static DB_POOL: OnceCell<PgPool> = OnceCell::new();

/// Initialize the DB connection pool with retry logic.
///
/// Respects env vars:
/// - `AXUM_DB_RETRY_COUNT` (default: 50)
/// - `AXUM_DB_RETRY_DELAY_SECS` (default: 1)
pub async fn init_database_with_retry_from_env() -> Result<()> {
    // ---
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let fname = "init_database_with_retry_from_env";

    if DB_POOL.get().is_some() {
        tracing::info!("{fname}: Pool is already initialized");
        return Ok(());
    }

    tracing::info!("🚨 axum-quickstart attaching to database at: {:?}", url);

    let retry_max = get_env_with_default!(u32, "AXUM_DB_RETRY_COUNT", 50);
    let delay_sec = get_env_with_default!(u64, "AXUM_DB_RETRY_DELAY_SECS", 1);

    for attempt in 1..=retry_max {
        // ---
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(delay_sec))
            .connect(&url)
            .await
        {
            Ok(pool) => {
                // ---
                if DB_POOL.set(pool).is_err() {
                    // ---

                    // This would happen only if this function is called from multiple
                    // threads concurrently which is not supposed to happen since it is
                    // called early in main, but we handle if it does by just dropping the
                    // new (2nd) one.

                    tracing::warn!("{fname}: Pool is already initialized");
                }
                return Ok(());
            }
            Err(e) if attempt == retry_max => {
                return Err(anyhow!(
                    "{fname}: Failed to connect to DB after {retry_max} retries: {e}"
                ));
            }
            Err(_) => {
                let backoff_secs = Duration::from_secs(std::cmp::min(2u64.pow(attempt - 1), 8));
                tracing::warn!(
                    "DB not ready (attempt {}/{}) — retrying in {}s...",
                    attempt,
                    retry_max,
                    backoff_secs.as_secs()
                );
                tokio::time::sleep(backoff_secs).await;
            }
        }
    }
    unreachable!("Exhausted retries should already have returned above")
}

pub fn create_postgres_repository() -> Result<RepositoryPtr> {
    // ---
    let pool = DB_POOL
        .get()
        .expect("Pool not initialized. Call init_pool_with_retry() first.");

    let rep = PostgresRepository::new(pool.clone());
    Ok(Arc::new(rep))
}

pub struct PostgresRepository {
    // ---
    pool: PgPool,
}

impl PostgresRepository {
    // ---
    pub fn new(pool: PgPool) -> Self {
        // ---
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Repository for PostgresRepository {
    // ---
    async fn create_user(&self, username: &str) -> Result<User> {
        // ---
        let user = User::new(username.to_string());

        sqlx::query("INSERT INTO users (id, username, created_at) VALUES ($1, $2, $3)")
            .bind(user.id)
            .bind(&user.username)
            .bind(user.created_at)
            .execute(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        // ---
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            created_at: r.created_at,
        }))
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        // ---
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, created_at FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            created_at: r.created_at,
        }))
    }

    async fn save_credential(&self, credential: Credential) -> Result<()> {
        // ---
        sqlx::query(
            "INSERT INTO credentials (id, user_id, public_key, counter, created_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&credential.id)
        .bind(credential.user_id)
        .bind(&credential.public_key)
        .bind(credential.counter)
        .bind(credential.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_credential_by_id(&self, credential_id: &[u8]) -> Result<Option<Credential>> {
        // ---
        let row = sqlx::query_as::<_, CredentialRow>(
            "SELECT id, user_id, public_key, counter, created_at
             FROM credentials WHERE id = $1",
        )
        .bind(credential_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Credential {
            id: r.id,
            user_id: r.user_id,
            public_key: r.public_key,
            counter: r.counter,
            created_at: r.created_at,
        }))
    }

    async fn get_credentials_by_user(&self, user_id: Uuid) -> Result<Vec<Credential>> {
        // ---
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT id, user_id, public_key, counter, created_at
             FROM credentials WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Credential {
                id: r.id,
                user_id: r.user_id,
                public_key: r.public_key,
                counter: r.counter,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn update_credential(&self, credential: Credential) -> Result<()> {
        // ---
        sqlx::query("UPDATE credentials SET public_key = $1, counter = $2 WHERE id = $3")
            .bind(&credential.public_key)
            .bind(credential.counter)
            .bind(&credential.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_credential(&self, credential_id: &[u8]) -> Result<()> {
        // ---
        sqlx::query("DELETE FROM credentials WHERE id = $1")
            .bind(credential_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
