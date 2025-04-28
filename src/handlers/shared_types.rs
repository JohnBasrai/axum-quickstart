use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Wrapper type for successful API responses.
///
/// Encapsulates the data payload and prepares it for JSON serialization.
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}
