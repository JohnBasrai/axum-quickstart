use crate::DatabaseConfig;
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

static DB_POOL: OnceCell<PgPool> = OnceCell::new();

/// Initialize the DB connection pool with retry logic.
///
/// Respects env vars:
/// - `AXUM_DB_RETRY_COUNT` (default: 50)
/// - `AXUM_DB_RETRY_DELAY_SECS` (default: 1)
pub async fn init_database_with_retry_from_env() -> Result<()> {
    // ---

    if DB_POOL.get().is_some() {
        tracing::debug!("init_database_with_retry_from_env: Pool is already initialized");
        return Ok(());
    }

    init_database_with_retry(&DatabaseConfig::from_env()?).await
}

async fn init_database_with_retry(cfg: &DatabaseConfig) -> Result<()> {
    // ---
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let fname = "init_database_with_retry";

    tracing::info!("🚨 axum-quickstart attaching to database at: {:?}", url);

    for attempt in 1..=cfg.retry_count {
        // ---
        match PgPoolOptions::new()
            .max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            .acquire_timeout(cfg.acquire_timeout)
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
            Err(e) if attempt == cfg.retry_count => {
                return Err(anyhow!(
                    "{fname}: Failed to connect to DB after {} retries: {e}",
                    cfg.retry_count
                ));
            }
            Err(_) => {
                let backoff_secs = Duration::from_secs(std::cmp::min(2u64.pow(attempt - 1), 8));
                tracing::warn!(
                    "DB not ready (attempt {}/{}) — retrying in {}s...",
                    attempt,
                    cfg.retry_count,
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
        tracing::debug!(
            "POOL STATE before test: size={}, idle={}",
            pool.size(),
            pool.num_idle()
        );

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

#[cfg(test)]
mod schema_tests {
    // ---
    // use super::*;
    use sqlx::PgPool;
    use std::env;
    use uuid::Uuid;

    async fn test_pool() -> PgPool {
        // ---

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set for database schema tests");

        tracing::info!("Connecting to DATABASE_URL:{database_url}");

        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test(flavor = "current_thread")]
    async fn users_delete_cascades_credentials() {
        // ---
        let pool = test_pool().await;

        let user_id = Uuid::new_v4();
        let credential_id: Vec<u8> = vec![1, 2, 3, 4];

        // Insert user (raw SQL)
        sqlx::query(
            "INSERT INTO users (id, username, created_at)
             VALUES ($1, $2, NOW())",
        )
        .bind(user_id)
        .bind("cascade_test_user")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

        // Insert credential (raw SQL)
        sqlx::query(
            "INSERT INTO credentials (id, user_id, public_key, counter, created_at)
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(&credential_id)
        .bind(user_id)
        .bind(vec![9u8, 9, 9]) // ✅ Vec<u8> explicit
        .bind(0_i32)
        .execute(&pool)
        .await
        .expect("Failed to insert credential");

        // Sanity check: credential exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM credentials WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count credentials");

        assert_eq!(count, 1);

        // Delete user (raw SQL)
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to delete user");

        // Verify cascade delete
        let remaining: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM credentials WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to verify cascade delete");

        assert_eq!(
            remaining, 0,
            "credentials should be deleted via ON DELETE CASCADE"
        );
    }
}
