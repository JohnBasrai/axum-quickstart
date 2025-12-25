//! Application state management.
//!
//! This module defines the shared state structure that gets passed to all
//! Axum handlers via the `State` extractor. The `AppState` contains shared
//! resources like database connections, metrics implementations, WebAuthn
//! configuration, and other application-wide dependencies.
//!
//! The state is designed to be cheaply cloneable (using `Arc` internally
//! where needed) so it can be passed efficiently to each request handler
//! without expensive copying of resources.

use crate::domain::{MetricsPtr, RepositoryPtr};
use axum::http::StatusCode;
use redis::Client;
use std::sync::Arc;
use std::time::Duration;
use webauthn_rs::Webauthn;

/// Shared application state passed to all Axum handlers.
///
/// This struct serves as the Dependency Injection container for the application.
/// It holds all shared resources needed by HTTP handlers and is cloned cheaply
/// for each request via Axum's `State` extractor.
///
/// # Design Principles
///
/// - **Dependency Inversion**: Handlers depend on abstractions (Repository trait),
///   not concrete implementations (PostgresRepository).
/// - **Immutable After Initialization**: State is built once at startup and
///   never mutated. Handlers create new connections/transactions per request.
/// - **Cheap Cloning**: All heavy resources (database pools, WebAuthn config)
///   are wrapped in `Arc`, making the struct efficiently cloneable.
///
/// # Lifecycle
///
/// 1. Created once in `create_router()` during application startup
/// 2. Attached to the Axum router via `.with_state(app_state)`
/// 3. Cloned automatically by Axum for each incoming HTTP request
/// 4. Handlers extract via `State(state): State<AppState>`
///
/// # Fields
///
/// - `redis_client`: Client for creating ephemeral Redis connections (challenges, sessions)
/// - `metrics`: Metrics implementation for observability (Prometheus or no-op)
/// - `repository`: Database abstraction for persistent storage (users, credentials)
/// - `webauthn`: WebAuthn protocol handler for passkey operations (registration, authentication)
/// - `challenge_ttl`: Time-to-live for WebAuthn challenges stored in Redis
#[derive(Clone)]
pub(crate) struct AppState {
    /// Redis client for creating multiplexed async connections on demand.
    ///
    /// Used for ephemeral data (WebAuthn challenges, session tokens, cache).
    /// Handlers call `get_conn()` to obtain a connection for each request.
    redis_client: Client,

    /// Metrics implementation for recording application events.
    ///
    /// Either Prometheus-backed (production) or no-op (testing/development).
    /// Wrapped in `Arc` via `MetricsPtr` for cheap cloning.
    metrics: MetricsPtr,

    /// Repository abstraction for persistent storage.
    ///
    /// Provides access to users and credentials via the `Repository` trait.
    /// Backed by PostgreSQL with SQLx connection pooling.
    /// Wrapped in `Arc` via `RepositoryPtr` for cheap cloning.
    repository: RepositoryPtr,

    /// WebAuthn protocol handler.
    ///
    /// Configured with relying party identity (RP ID, origin, name).
    /// Used for generating challenges and verifying credentials.
    /// Wrapped in `Arc` because `Webauthn` does not implement `Clone`.
    webauthn: Arc<Webauthn>,

    /// Time-to-live for WebAuthn challenges in Redis.
    ///
    /// Challenges expire after this duration to prevent replay attacks.
    /// Typically 5 minutes (300 seconds).
    challenge_ttl: Duration,
}

impl AppState {
    // ---

    pub fn new(
        redis_client: Client,
        metrics: MetricsPtr,
        repository: RepositoryPtr,
        webauthn: Arc<Webauthn>,
        challenge_ttl: Duration,
    ) -> Self {
        // ---
        AppState {
            redis_client,
            metrics,
            repository,
            webauthn,
            challenge_ttl,
        }
    }

    /// Creates a new multiplexed Redis connection.
    ///
    /// Logs an error if connection fails and returns HTTP 500.
    pub(crate) async fn get_conn(&self) -> Result<redis::aio::MultiplexedConnection, StatusCode> {
        // ---
        self.redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                tracing::error!("Failed to connect to Redis: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    /// Get a reference to the metrics implementation.
    pub(crate) fn metrics(&self) -> &MetricsPtr {
        // ---
        &self.metrics
    }

    /// Get a reference to the repository implementation.
    pub(crate) fn repository(&self) -> &RepositoryPtr {
        // ---
        &self.repository
    }

    /// Get a reference to the WebAuthn instance.
    pub(crate) fn webauthn(&self) -> &Webauthn {
        // ---
        &self.webauthn
    }

    /// Get the WebAuthn challenge TTL.
    pub(crate) fn challenge_ttl(&self) -> Duration {
        // ---
        self.challenge_ttl
    }
}

#[cfg(test)]
mod tests {
    // ---

    use super::*;
    use crate::config::WebAuthnConfig;
    use crate::create_webauthn;
    use crate::domain::{Credential, Repository, User};
    use crate::infrastructure::create_noop_metrics;
    use anyhow::Result;
    use uuid::Uuid;

    // Mock repository for unit tests - not used, just satisfies AppState requirements
    struct MockRepository;

    #[async_trait::async_trait]
    impl Repository for MockRepository {
        // ---

        async fn create_user(&self, _username: &str) -> Result<User> {
            unimplemented!("Mock repository - not used in AppState unit tests")
        }
        async fn get_user_by_username(&self, _username: &str) -> Result<Option<User>> {
            unimplemented!()
        }
        async fn get_user_by_id(&self, _user_id: Uuid) -> Result<Option<User>> {
            unimplemented!()
        }
        async fn save_credential(&self, _credential: Credential) -> Result<()> {
            unimplemented!()
        }
        async fn get_credentials_by_user(&self, _user_id: Uuid) -> Result<Vec<Credential>> {
            unimplemented!()
        }
        async fn get_credential_by_id(&self, _credential_id: &[u8]) -> Result<Option<Credential>> {
            unimplemented!()
        }
        async fn update_credential(&self, _credential: Credential) -> Result<()> {
            unimplemented!()
        }
        async fn delete_credential(&self, _credential_id: &[u8]) -> Result<()> {
            unimplemented!()
        }
    }

    fn test_webauthn_config() -> WebAuthnConfig {
        // ---
        WebAuthnConfig {
            rp_id: "localhost".to_string(),
            rp_name: "Test App".to_string(),
            origin: "http://localhost:8080".to_string(),
        }
    }

    #[test]
    fn test_app_state_creation_and_clone() {
        // ---
        // Test basic creation and that Clone works
        let redis_client = Client::open("redis://127.0.0.1:6379").unwrap();
        let metrics = create_noop_metrics().unwrap();
        let repository = Arc::new(MockRepository);
        let webauthn_config = test_webauthn_config();
        let webauthn = Arc::new(create_webauthn(&webauthn_config).unwrap());
        let challenge_ttl = Duration::from_secs(300);

        let app_state = AppState::new(redis_client, metrics, repository, webauthn, challenge_ttl);
        let _cloned = app_state.clone();

        // Verify accessors work
        let _metrics_ref = app_state.metrics();
        let _repo_ref = app_state.repository();
        let _webauthn_ref = app_state.webauthn();
        assert_eq!(app_state.challenge_ttl(), Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_redis_connection_failure() {
        // ---
        // Test that connection failures return proper error
        let redis_client = Client::open("redis://invalid-host:6379").unwrap();
        let metrics = create_noop_metrics().unwrap();
        let repository = Arc::new(MockRepository);
        let webauthn_config = test_webauthn_config();
        let webauthn = Arc::new(create_webauthn(&webauthn_config).unwrap());
        let challenge_ttl = Duration::from_secs(300);

        let app_state = AppState::new(redis_client, metrics, repository, webauthn, challenge_ttl);

        let result = app_state.get_conn().await;
        assert_eq!(result.unwrap_err(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
