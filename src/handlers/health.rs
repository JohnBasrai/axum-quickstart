use axum::{Json, extract::{Query, State}, http::StatusCode};
use serde::Deserialize;
use crate::AppState;
use redis::AsyncCommands;

#[derive(serde::Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

#[derive(Deserialize)]
pub struct HealthQuery {
    mode: Option<String>,
}

/// Responds with the health status of the server.
///
/// - By default (no query parameters), performs a light check to confirm the web server
///   is running.
///
/// - If `mode=full` is passed as a query parameter, also pings the Redis backend to
///   verify database connectivity.
///
/// # Query Parameters
/// - `mode`: Optional. Accepts `"light"` (default) or `"full"`.
///
/// # Responses
/// - `200 OK` with `{ "status": "ok" }` if server (and Redis, in full mode) are healthy.
/// - `500 INTERNAL SERVER ERROR` with `{ "status": "error" }` if Redis connection or ping fails in full mode.
///
/// # Examples
/// - `GET /health` → 200 OK
/// - `GET /health?mode=full` → 200 OK or 500 INTERNAL SERVER ERROR
pub async fn health_check(
    State(state): State<AppState>,
    Query(params): Query<HealthQuery>,
) -> (StatusCode, Json<HealthResponse>) {
    match params.mode.as_deref() {
        Some("full") => {
            // Full health check: Ping Redis
            let mut conn = match state.get_conn().await {
                Ok(conn) => conn,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(HealthResponse { status: "error" })
                    )
                }
            };

            let ping_result: redis::RedisResult<String> = conn.ping().await;
            match ping_result {
                Ok(_) => (
                    StatusCode::OK,
                    Json(HealthResponse { status: "ok" })
                ),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HealthResponse { status: "error" })
                ),
            }
        }
        _ => {
            // Light health check
            (
                StatusCode::OK,
                Json(HealthResponse { status: "ok" })
            )
        }
    }
}
