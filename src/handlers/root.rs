use crate::AppState;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use std::time::Instant;

/// Handler for the root endpoint (GET /).
///
/// Returns an HTML page with information about the API, including:
/// - Application version from Cargo.toml
/// - List of available endpoints
/// - Basic styling for a clean presentation
///
/// This serves as both a landing page and API documentation for users
/// accessing the service through a web browser.
pub async fn root_handler(State(state): State<AppState>) -> impl IntoResponse {
    let start = Instant::now();
    let version = env!("CARGO_PKG_VERSION");

    let html = Html(format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>AXUM Quickstart</title>
  <style>
    body {{
      font-family: sans-serif;
      background-color: #f9f9f9;
      margin: 2rem;
      color: #222;
    }}
    .container {{
      background-color: white;
      padding: 2rem;
      border-radius: 8px;
      max-width: 700px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    }}
    h1 {{
      font-size: 2em;
      margin-bottom: 0.25em;
    }}
    p.version {{
      margin-top: 0;
      font-weight: bold;
    }}
    pre {{
      background: #f4f4f4;
      padding: 1em;
      overflow-x: auto;
      border-radius: 6px;
    }}
    code {{
      font-family: monospace;
    }}
  </style>
</head>
<body>
  <div class="container">
    <h1>AXUM Quickstart — Movie API 👋</h1>
    <p class="version">Version: {version}</p>
    <p>
      Welcome to the Movie API demo. This project showcases CRUD operations,
      health checks (including Redis), and dynamic version reporting.
    </p>
    <pre><code>
Available endpoints:
  - GET    /movies/get/{{id}}     Fetch a movie by ID
  - POST   /movies/add            Add a new movie entry
  - PUT    /movies/update         Update a movie entry by ID
  - DELETE /movies/delete         Delete a movie entry by ID
  - GET    /health                Light health check
  - GET    /health?mode=full      Full health check (includes Redis)
  - GET    /metrics               Prometheus metrics endpoint
    </code></pre>
  </div>
</body>
</html>
"#
    ));

    // Record metrics for the root handler
    state.metrics().record_http_request(start, "/", "GET", 200);

    html
}
