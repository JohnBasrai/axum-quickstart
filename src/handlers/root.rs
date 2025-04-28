use axum::response::IntoResponse;

pub async fn root_handler() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"Welcome to the Movie API 👋
Version: {version}

Available endpoints:
  - GET    /movies/get/{{id}} - Fetch a movie by ID
  - POST   /movies/add        - Add a movie entry
  - PUT    /movies/update     - Update a movie entry by id
  - DELETE /movies/delete     - Delete a movie entry by id
  - GET    /health            - Light health check
  - GET    /health?mode=full  - Full health check (includes Redis)

This API demonstrates CRUD operations for movies, health checking (including Redis connectivity),
and dynamic version reporting.
"#
    )
}
