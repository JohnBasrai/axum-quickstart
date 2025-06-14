use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus recorder globally and store the handle.
pub fn init_metrics() {
    let handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    HANDLE
        .set(handle)
        .expect("metrics recorder already initialized");
}

/// Render the current metrics in Prometheus text format.
pub fn render_metrics() -> String {
    HANDLE
        .get()
        .expect("metrics recorder not initialized")
        .render()
}
