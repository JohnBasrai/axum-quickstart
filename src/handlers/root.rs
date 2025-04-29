use axum::{
    response::{Html, IntoResponse},
};

pub async fn root_handler() -> impl IntoResponse {
    // ---
    let version = env!("CARGO_PKG_VERSION");

    Html(format!(r#"
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
    </code></pre>
  </div>
</body>
</html>
"#, version = version))
}
