use anyhow::Result;
use axum_quickstart::create_app;
use reqwest::StatusCode;
use std::net::SocketAddr;
use tokio::task::JoinHandle;

async fn spawn_app() -> Result<(SocketAddr, JoinHandle<()>)> {
    // ---
    let app = create_app()?;
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let server = axum::serve(listener, app.into_make_service());

    let handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Server error: {:?}", e);
        }
    });

    Ok((addr, handle))
}

#[tokio::test]
async fn health_check_works() -> Result<()> {
    // ---
    let (addr, _server_handle) = spawn_app().await?;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("http://{}/health", addr))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.json::<serde_json::Value>().await?;
    assert_eq!(body["status"], "ok");

    Ok(())
}

#[tokio::test]
async fn health_check_full_mode_works() -> Result<()> {
    // ---
    let (addr, _server_handle) = spawn_app().await?;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/health?mode=full", addr))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.json::<serde_json::Value>().await?;
    assert_eq!(body["status"], "ok");

    Ok(())
}

// Test Full happy path lifecycle (add, get, update, delete, verify 404)
#[tokio::test]
async fn movie_lifecycle_works() -> Result<()> {
    // ---
    use rand::Rng; // add rand = "0.8" to [dev-dependencies] if needed
    let (addr, _server_handle) = spawn_app().await?;
    let client = reqwest::Client::new();

    // Create a unique movie title
    let suffix: u32 = rand::thread_rng().gen();
    let unique_title = format!("The Shawshank Redemption TEST-{suffix}");

    // 1. Add movie
    let new_movie = serde_json::json!({
        "title": unique_title,
        "year": 1994,
        "stars": 4.5
    });

    let response = client
        .post(format!("http://{addr}/movies/add"))
        .json(&new_movie)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.json::<serde_json::Value>().await?;
    let movie_id = body["id"].as_str().expect("Missing 'id'").to_string();

    // 2. Fetch movie
    let response = client
        .get(format!("http://{addr}/movies/get/{movie_id}"))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Update movie
    let updated_movie = serde_json::json!({
        "title": format!("{} (Director's Cut)", unique_title),
        "year": 1994,
        "stars": 4.8
    });

    let response = client
        .put(format!("http://{addr}/movies/update/{movie_id}"))
        .json(&updated_movie)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Fetch updated movie
    let response = client
        .get(format!("http://{addr}/movies/get/{movie_id}"))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Delete movie
    let response = client
        .delete(format!("http://{addr}/movies/delete/{movie_id}"))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. Confirm fetch gives 404 after it was deleted
    let response = client
        .get(format!("http://{addr}/movies/get/{movie_id}"))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn add_movie_missing_title_returns_422() -> Result<()> {
    // ---
    let (addr, _server_handle) = spawn_app().await?;
    let client = reqwest::Client::new();

    let bad_movie = serde_json::json!({
        "year": 1994,
        "stars": 4.5
        // Missing "title"
    });

    let response = client
        .post(format!("http://{addr}/movies/add"))
        .json(&bad_movie)
        .send()
        .await?;

    // AXUM framework parses the json and it returns 422 fo rmissing fields.
    //
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    Ok(())
}

#[tokio::test]
async fn add_movie_with_invalid_year_returns_400() -> Result<()> {
    // ---
    let (addr, _server_handle) = spawn_app().await?;
    let client = reqwest::Client::new();

    let invalid_movie = serde_json::json!({
        "title": "Bad Year Movie",
        "year": 1500, // valid u16, but unrealistic for a movie
        "stars": 4.5
    });

    let response = client
        .post(format!("http://{addr}/movies/add"))
        .json(&invalid_movie)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn add_movie_with_invalid_stars_returns_400() -> Result<()> {
    // ---
    let (addr, _server_handle) = spawn_app().await?;
    let client = reqwest::Client::new();

    let invalid_movie = serde_json::json!({
        "title": "Bad Stars Movie",
        "year": 1994,
        "stars": 6.5 // Invalid, should be between 0.0 and 5.0
    });

    let response = client
        .post(format!("http://{addr}/movies/add"))
        .json(&invalid_movie)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn fetch_nonexistent_movie_returns_404() -> Result<()> {
    // ---
    let (addr, _server_handle) = spawn_app().await?;
    let client = reqwest::Client::new();

    let fake_id = "nonexistent123";

    let response = client
        .get(format!("http://{addr}/movies/get/{fake_id}"))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}
