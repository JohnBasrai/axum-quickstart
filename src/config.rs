// src/config.rs

//! Application configuration loaded from environment variables.
//!
//! This module defines all startup-time configuration for the service.
//! Configuration is validated eagerly and failures are treated as
//! deployment errors rather than recoverable runtime conditions.

use anyhow::Result;
use std::time::Duration;

// ============================================================
// Local macros (config-only, intentionally explicit)
// ============================================================

/// Reads a required environment variable.
///
/// # Behavior
/// - Fails fast if the variable is missing
/// - Produces a clear, human-readable error message
/// - Intended for startup-time configuration validation
///
/// Missing configuration is treated as a deployment error,
/// not a recoverable runtime condition.
macro_rules! required_env {
    // ---
    ($key:literal) => {
        std::env::var($key)
            .map_err(|_| anyhow::anyhow!(concat!("Missing required configuration: ", $key)))?
    };
}

/// Reads an optional environment variable and attempts to parse it.
///
/// If the variable is missing or cannot be parsed, the provided
/// default value is used. This macro is appropriate for non-critical
/// tuning parameters where fallback behavior is acceptable.
macro_rules! optional_env_parse {
    // ---
    ($key:literal, $ty:ty, $default:expr) => {
        std::env::var($key)
            .ok()
            .and_then(|v| v.parse::<$ty>().ok())
            .unwrap_or($default)
    };
}

#[cfg(test)]
/// Asserts that a configuration constructor fails due to a missing
/// required environment variable.
///
/// This macro is intended for config unit tests only and enforces
/// consistent error messages across failure cases.
macro_rules! assert_missing_config {
    // ---
    ($expr:expr, $key:literal) => {{
        let err = $expr.expect_err("expected configuration error");
        assert!(
            err.to_string()
                .contains(concat!("Missing required configuration: ", $key)),
            "unexpected error: {err}"
        );
    }};
}

// ============================================================
// Public configuration facade
// ============================================================

/// Aggregated application configuration.
///
/// This is the single source of truth for startup configuration.
/// All required configuration is validated eagerly during initialization.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: database::DatabaseConfig,
    pub redis: redis::RedisConfig,
    pub webauthn: webauthn::WebAuthnConfig,
}

impl AppConfig {
    /// Loads and validates all application configuration from the environment.
    ///
    /// # Errors
    /// Returns an error if any required configuration is missing or invalid.
    /// This function is intended to be called exactly once at startup.
    pub fn from_env() -> Result<Self> {
        // ---
        Ok(Self {
            database: database::DatabaseConfig::from_env()?,
            redis: redis::RedisConfig::from_env()?,
            webauthn: webauthn::WebAuthnConfig::from_env()?,
        })
    }
}

// ============================================================
// Database configuration
// ============================================================

mod database {
    // ---
    use super::*;

    /// Database-related configuration derived from environment variables.
    ///
    /// This configuration is required for the service to function and
    /// is validated eagerly during startup.
    #[derive(Debug, Clone)]
    pub struct DatabaseConfig {
        /// PostgreSQL connection string.
        pub database_url: String,

        /// Number of retry attempts when initializing the database connection. Defaults to 50.
        pub retry_count: u32,

        /// Maximum time to wait when acquiring a connection from the pool. Defaults to 30 seconds.
        pub acquire_timeout: Duration,

        /// Minimum number of connections to keep in the pool, even when idle. Defaults to 2.
        pub min_connections: u32,

        /// Minimum number of connections to be open concurrently. Defaults to 15
        pub max_connections: u32,
    }

    impl DatabaseConfig {
        /// Builds a [`DatabaseConfig`] from environment variables.
        ///
        /// # Errors
        /// Returns an error if required configuration is missing.
        /// Startup will fail fast rather than continuing with incomplete
        /// or invalid configuration.
        pub fn from_env() -> Result<Self> {
            // ---
            let database_url = required_env!("DATABASE_URL");
            let retry_count = optional_env_parse!("AXUM_DB_RETRY_COUNT", u32, 50);
            let acquire_timeout_secs = optional_env_parse!("AXUM_DB_ACQUIRE_TIMEOUT_SEC", u64, 30);
            let min_connections = optional_env_parse!("AXUM_DB_MIN_CONNECTIONS", u32, 2);
            let max_connections = optional_env_parse!("AXUM_DB_MAX_CONNECTIONS", u32, 15);

            Ok(Self {
                database_url,
                retry_count,
                acquire_timeout: Duration::from_secs(acquire_timeout_secs),
                min_connections,
                max_connections,
            })
        }
    }
}
pub use database::DatabaseConfig;

// ============================================================
// Redis configuration
// ============================================================

mod redis {
    // ---
    use super::*;

    /// Redis-related configuration used for ephemeral and cache-backed state.
    ///
    /// In Phase 2, Redis is used to store WebAuthn challenges with a
    /// bounded time-to-live.
    #[derive(Debug, Clone)]
    pub struct RedisConfig {
        /// Redis connection string.
        pub url: String,

        /// Time-to-live for WebAuthn challenge data.
        pub webauthn_challenge_ttl: Duration,
    }

    impl RedisConfig {
        /// Builds a [`RedisConfig`] from environment variables.
        ///
        /// # Errors
        /// Returns an error if required configuration is missing.
        pub fn from_env() -> Result<Self> {
            // ---
            let url = required_env!("AXUM_REDIS_URL");

            let ttl_secs = optional_env_parse!("AXUM_WEBAUTHN_CHALLENGE_TTL_SEC", u64, 300);

            Ok(Self {
                url,
                webauthn_challenge_ttl: Duration::from_secs(ttl_secs),
            })
        }
    }
}
pub use redis::RedisConfig;

// ============================================================
// WebAuthn configuration
// ============================================================

mod webauthn {
    // ---
    use super::*;

    /// WebAuthn / Passkeys configuration.
    ///
    /// These values define the relying party identity and security
    /// origin used during WebAuthn registration and authentication.
    #[derive(Debug, Clone)]
    pub struct WebAuthnConfig {
        /// Relying Party ID (typically a domain name).
        pub rp_id: String,

        /// Human-readable Relying Party name.
        pub rp_name: String,

        /// Fully-qualified origin (e.g. https://example.com).
        pub origin: String,
    }

    impl WebAuthnConfig {
        /// Builds a [`WebAuthnConfig`] from environment variables.
        ///
        /// # Errors
        /// Returns an error if required configuration is missing.
        /// WebAuthn configuration is considered security-critical
        /// and must be explicitly provided.
        pub fn from_env() -> Result<Self> {
            // ---
            let rp_id = required_env!("AXUM_WEBAUTHN_RP_ID");
            let origin = required_env!("AXUM_WEBAUTHN_ORIGIN");

            let rp_name = std::env::var("AXUM_WEBAUTHN_RP_NAME")
                .unwrap_or_else(|_| "Axum Quickstart".to_string());

            Ok(Self {
                rp_id,
                rp_name,
                origin,
            })
        }
    }
}
//pub use webauthn::WebAuthnConfig;

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    // ---
    use super::*;
    use anyhow::Result;
    use serial_test::serial;

    #[test]
    #[serial]
    fn missing_database_url_fails() -> Result<()> {
        // ---
        std::env::remove_var("DATABASE_URL");

        assert_missing_config!(database::DatabaseConfig::from_env(), "DATABASE_URL");

        Ok(())
    }

    #[test]
    #[serial]
    fn database_defaults_applied() -> Result<()> {
        // ---
        let db_url = "postgres://test";
        std::env::set_var("DATABASE_URL", db_url); // requried

        std::env::remove_var("AXUM_DB_RETRY_COUNT");
        std::env::remove_var("AXUM_DB_ACQUIRE_TIMEOUT_SEC");
        std::env::remove_var("AXUM_DB_MIN_CONNECTIONS");
        std::env::remove_var("AXUM_DB_MAX_CONNECTIONS");

        let cfg = database::DatabaseConfig::from_env()?;
        assert_eq!(cfg.database_url, db_url);
        assert_eq!(cfg.retry_count, 50);
        assert_eq!(cfg.acquire_timeout.as_secs(), 30);
        assert_eq!(cfg.min_connections, 2);
        assert_eq!(cfg.max_connections, 15);

        Ok(())
    }

    #[test]
    #[serial]
    fn database_overrides_defaults() -> Result<()> {
        // ---

        let db_url = "postgres://test";
        std::env::set_var("DATABASE_URL", db_url);
        std::env::set_var("AXUM_DB_RETRY_COUNT", "3");
        std::env::set_var("AXUM_DB_ACQUIRE_TIMEOUT_SEC", "5");
        std::env::set_var("AXUM_DB_MIN_CONNECTIONS", "10");
        std::env::set_var("AXUM_DB_MAX_CONNECTIONS", "1000");

        let cfg = database::DatabaseConfig::from_env()?;
        assert_eq!(cfg.retry_count, 3);
        assert_eq!(cfg.acquire_timeout.as_secs(), 5);
        assert_eq!(cfg.database_url, db_url);
        assert_eq!(cfg.min_connections, 10);
        assert_eq!(cfg.max_connections, 1000);

        Ok(())
    }

    #[test]
    #[serial]
    fn app_config_from_env_success() -> Result<()> {
        // ---
        std::env::set_var("DATABASE_URL", "postgres://test");
        std::env::set_var("AXUM_REDIS_URL", "redis://localhost");
        std::env::set_var("AXUM_WEBAUTHN_RP_ID", "example.com");
        std::env::set_var("AXUM_WEBAUTHN_ORIGIN", "https://example.com");

        let cfg = AppConfig::from_env()?;
        assert_eq!(cfg.webauthn.rp_name, "Axum Quickstart");

        Ok(())
    }
}
