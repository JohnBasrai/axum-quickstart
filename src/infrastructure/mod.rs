pub mod metrics;

// Re-export the factory functions for easy access
pub use metrics::{create_noop_metrics, create_prom_metrics};
