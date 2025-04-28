use anyhow::Result;
use reqwest::StatusCode;

// Import your server app
use axum_quickstart::create_app; // adjust path to your `create_app` function

use std::net::SocketAddr;
use tokio::task::JoinHandle;

async fn spawn_app() -> Result<(SocketAddr, JoinHandle<()>)> {
    // ---
    let app = create_app()?; // <-- no expect, just ?
    
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

