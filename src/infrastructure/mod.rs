mod database;
pub mod metrics;

// Re-export the factory functions for easy access
pub use database::create_postgres_repository;
pub use metrics::{create_noop_metrics, create_prom_metrics};
