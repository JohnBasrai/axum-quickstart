use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};

/// Handler for the `/metrics` endpoint.
///
/// Returns metrics in Prometheus text format for scraping.
/// Uses the metrics implementation from AppState, which could be
/// either Prometheus or no-op depending on configuration.
pub async fn metrics_handler(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // ---

    let metrics_text = app_state.metrics().render();

    Ok((
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics_text,
    ))
}
