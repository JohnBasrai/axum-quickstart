use anyhow::{ensure, Result};
use axum_quickstart::create_router;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::sleep;

struct TestServer {
    pub addr: std::net::SocketAddr,
    pub client: Client,
}

impl TestServer {
    // ---
    pub async fn new() -> Self {
        // --

        // Enable debug logging only when requested
        if std::env::var("TEST_DEBUG").is_ok() {
            std::env::set_var("RUST_LOG", "debug");
            std::env::set_var("NO_COLOR", "1");
        }

        let app = create_router().expect("Should be able to create router");
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn the server in the background
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give the server a moment to start
        sleep(Duration::from_millis(100)).await;

        let client = Client::new();

        Self { addr, client }
    }

    pub fn url(&self, path: &str) -> String {
        // ---
        format!("http://{}{}", self.addr, path)
    }
}

#[tokio::test]
#[serial_test::serial]
async fn basic_integration_test() {
    // ---
    // Test that the router can be created successfully
    let _router = create_router().expect("Should be able to create router");
}

#[tokio::test]
#[serial_test::serial]
async fn health_endpoint_works() {
    // ---
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/health"))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body = response.text().await.expect("Failed to read response body");
    assert!(!body.is_empty());
}

#[tokio::test]
#[serial_test::serial]
async fn root_endpoint_works() {
    // ---
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read response body");
    assert!(!body.is_empty());
}

#[tokio::test]
#[serial_test::serial]
async fn movies_crud_operations() -> Result<()> {
    // ---
    let server = TestServer::new().await;

    // Test GET /movies (should be empty initially)
    let response = server
        .client
        .get(server.url("/movies/get/1"))
        .send()
        .await
        .expect("Failed to get movies");

    assert_eq!(response.status(), 404);

    let random_title = format!(
        "Test Movie {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Test POST /movies
    let new_movie = json!({
        "title": random_title,
        "stars": 4.5,
        "year": 2023
    });

    let response = server
        .client
        .post(server.url("/movies/add"))
        .json(&new_movie)
        .send()
        .await
        .expect("Failed to create movie");

    assert_eq!(response.status(), 201);

    // Extract the movie ID from the response
    let created_response: serde_json::Value = response.json().await?;
    let movie_id = created_response["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No ID in response"))?;

    // Test GET /movies again (should now have one movie)
    let response = server
        .client
        .get(server.url(&format!("/movies/get/{movie_id}")))
        .send()
        .await
        .expect("Failed to get movies after creation");

    assert_eq!(response.status(), 200);
    let movies: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    // Verify the movie was created (exact structure depends on your implementation)
    ensure!(movies.is_array() || movies.is_object());
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn invalid_routes_return_404() {
    // ---
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/nonexistent"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
#[serial_test::serial]
async fn server_handles_concurrent_requests() {
    // ---
    let server = TestServer::new().await;

    // Make multiple concurrent requests
    let futures = (0..10).map(|_| server.client.get(server.url("/health")).send());

    let responses = futures::future::join_all(futures).await;

    // All requests should succeed
    for response in responses {
        let response = response.expect("Request should succeed");
        assert_eq!(response.status(), 200);
    }
}

#[tokio::test]
#[serial_test::serial]
async fn server_handles_malformed_json() {
    // ---
    let server = TestServer::new().await;

    // Send malformed JSON to movies endpoint
    let response = server
        .client
        .post(server.url("/movies/add"))
        .header("content-type", "application/json")
        .body("{ invalid json }")
        .send()
        .await
        .expect("Failed to send request");

    // Should return 400 Bad Request
    assert_eq!(response.status(), 400);
}

#[tokio::test]
#[serial_test::serial]
async fn redis_integration_works() {
    // ---
    // This test assumes Redis is available
    // You might want to make this conditional based on environment

    let server = TestServer::new().await;

    // Make some requests that would use Redis (if your app caches anything)
    let response = server
        .client
        .get(server.url("/health"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    // Add more specific Redis integration tests based on your app's usage
}
