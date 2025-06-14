pub mod noop;
pub mod prometheus;

// Re-export the factory functions for easy access
pub use noop::create as create_noop_metrics;
pub use prometheus::create as create_prom_metrics;
