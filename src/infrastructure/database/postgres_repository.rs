use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Credential, Repository, User};

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

pub fn create_postgres_repository(pool: PgPool) -> impl Repository {
    // ---
    PostgresRepository::new(pool)
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
