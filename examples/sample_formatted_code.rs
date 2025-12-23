//! # Axum Quickstart Formatting Style Guide
//!
//! This example demonstrates the code formatting conventions used in this project.
//! It shows the proper use of `// ---` comment separators for visual clarity.
//!
//! Run with: `cargo run --example formatting_style_guide`

use std::sync::OnceLock;

// Mock types for demonstration (these don't need to actually work)
trait Metrics {
    fn render(&self) -> String;
    fn increment_counter(&self, name: &str, value: u64);
    fn record_histogram(&self, name: &str, value: f64);
}

static HANDLE: OnceLock<String> = OnceLock::new();

/// Initialize the metrics system (example function)
pub fn init_metrics() {
    // ---
    let handle = "mock_prometheus_handle".to_string();

    HANDLE
        .set(handle)
        .expect("metrics recorder already initialized");
}

/// Render the current metrics (example function)
pub fn render_metrics() -> String {
    // ---
    HANDLE
        .get()
        .unwrap_or(&"not_initialized".to_string())
        .clone()
}

/// Example struct demonstrating our formatting conventions
#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    name: String,
    enabled: bool,
}

impl PrometheusMetrics {
    // ---
    pub fn new(name: String) -> Self {
        // ---
        Self {
            name,
            enabled: true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        // ---
        self.enabled
    }

    pub fn enable(&mut self) {
        // ---
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        // ---
        self.enabled = false;
    }

    pub fn get_name(&self) -> &str {
        // ---
        &self.name
    }
}

impl Metrics for PrometheusMetrics {
    // ---
    fn render(&self) -> String {
        // ---
        if self.enabled {
            render_metrics()
        } else {
            String::from("# Metrics disabled")
        }
    }

    fn increment_counter(&self, name: &str, value: u64) {
        // ---
        if self.enabled {
            println!("Counter '{name}' incremented by {value}");
        }
    }

    fn record_histogram(&self, name: &str, value: f64) {
        // ---
        if self.enabled {
            println!("Histogram '{name}' recorded value {value}");
        }
    }
}

impl Default for PrometheusMetrics {
    // ---
    fn default() -> Self {
        // ---
        Self::new("default".to_string())
    }
}

// Example module demonstrating module formatting
pub mod helpers {
    // ---
    #[allow(unused_imports)] // For style guide demonstration
    use super::*;

    pub fn format_metric_name(name: &str) -> String {
        // ---
        name.replace('-', "_").to_lowercase()
    }

    pub fn validate_metric_value(value: f64) -> Result<f64, String> {
        // ---
        if value.is_finite() {
            Ok(value)
        } else {
            Err("Metric value must be finite".to_string())
        }
    }
}

// Private module for internal utilities
mod internal {
    // ---
    use std::collections::HashMap;

    #[allow(dead_code)] // For style guide demonstration
    pub(super) struct MetricCache {
        cache: HashMap<String, f64>,
    }

    #[allow(dead_code)] // For style guide demonstration
    impl MetricCache {
        // ---
        pub fn new() -> Self {
            // ---
            Self {
                cache: HashMap::new(),
            }
        }

        pub fn get(&self, key: &str) -> Option<f64> {
            // ---
            self.cache.get(key).copied()
        }

        pub fn set(&mut self, key: String, value: f64) {
            // ---
            self.cache.insert(key, value);
        }
    }
}

// Example functions showing different formatting patterns
pub fn complex_function_with_many_parameters(
    first_param: String,
    second_param: u64,
    third_param: bool,
    fourth_param: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    // ---
    let result = if third_param {
        format!("{first_param}-{second_param}")
    } else {
        "disabled".to_string()
    };

    match fourth_param {
        Some(override_value) => Ok(override_value),
        None => Ok(result),
    }
}

pub fn simple_function() -> u32 {
    // ---
    42
}

fn main() {
    // ---
    println!("🎨 Axum Quickstart Formatting Style Guide");
    println!("==========================================");

    // Initialize metrics
    init_metrics();

    // Create a metrics instance
    let metrics = PrometheusMetrics::new("example".to_string());

    // Demonstrate the API
    println!("✅ Metrics enabled: {}", metrics.is_enabled());
    println!("📊 Metrics name: {}", metrics.get_name());
    println!("📊 Metrics output:\n{}", metrics.render());

    // Demonstrate the methods
    metrics.increment_counter("example_counter", 5);
    metrics.record_histogram("example_histogram", 42.5);

    // Test helper functions
    let formatted_name = helpers::format_metric_name("test-metric-name");
    println!("🔧 Formatted metric name: {formatted_name}");

    // Test validation
    match helpers::validate_metric_value(42.5) {
        Ok(value) => println!("✅ Valid metric value: {value}"),
        Err(error) => println!("❌ Invalid metric value: {error}"),
    }

    // Test complex function
    match complex_function_with_many_parameters(
        "test".to_string(),
        123,
        true,
        Some("override".to_string()),
    ) {
        Ok(result) => println!("🎯 Complex function result: {result}"),
        Err(error) => println!("❌ Complex function error: {error}"),
    }

    println!("🎉 Style guide example completed!");
}

#[cfg(test)]
mod tests {
    // ---
    use super::*;

    #[test]
    fn test_prometheus_metrics_creation() {
        // ---
        let metrics = PrometheusMetrics::new("test".to_string());
        assert_eq!(metrics.get_name(), "test");
        assert!(metrics.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        // ---
        let mut metrics = PrometheusMetrics::new("test".to_string());

        metrics.disable();
        assert!(!metrics.is_enabled());

        metrics.enable();
        assert!(metrics.is_enabled());
    }

    mod helper_tests {
        // ---
        use super::super::helpers::*;

        #[test]
        fn test_format_metric_name() {
            // ---
            assert_eq!(format_metric_name("test-metric"), "test_metric");
            assert_eq!(format_metric_name("TEST-METRIC"), "test_metric");
        }

        #[test]
        fn test_validate_metric_value() {
            // ---
            assert!(validate_metric_value(42.0).is_ok());
            assert!(validate_metric_value(f64::INFINITY).is_err());
            assert!(validate_metric_value(f64::NAN).is_err());
        }
    }
}
