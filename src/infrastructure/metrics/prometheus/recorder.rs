use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus recorder globally and store the handle.
/// This function is safe to call multiple times - it will only initialize once.
/// Returns true if initialization was successful, false if already initialized.
pub fn init_metrics() -> bool {
    HANDLE.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install Prometheus recorder")
    });
    true
}

/// Render the current metrics in Prometheus text format.
pub fn render_metrics() -> String {
    HANDLE
        .get()
        .expect("metrics recorder not initialized")
        .render()
}
