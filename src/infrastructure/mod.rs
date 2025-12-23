mod database;
pub mod metrics;

// Re-export the factory functions for easy access
pub use database::postgres_repository::{
    create_postgres_repository, init_database_with_retry_from_env,
};
pub use metrics::{create_noop_metrics, create_prom_metrics};
