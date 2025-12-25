// Test helpers are intentionally partially used
#![allow(dead_code)]

use axum_quickstart::create_router;
use axum_quickstart::domain::init_database_with_retry_from_env;
use reqwest::Client;
use std::sync::Once;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::sleep;

macro_rules! set_env_if_unset {
    // ---
    ($key:expr, $val:expr) => {
        if std::env::var($key).is_err() {
            std::env::set_var($key, $val);
        }
    };
}

static INIT: Once = Once::new();

// ============================================================================
// Test Setup
// ============================================================================

/// Initialize test environment (database, Redis, env vars) variables once
pub async fn setup_test_env() {
    // ---
    // Set required environment variables for testing
    INIT.call_once(|| {
        // ---
        set_env_if_unset!(
            "DATABASE_URL",
            "postgres://postgres:postgres@localhost:5432/axum_quickstart_test"
        );
        set_env_if_unset!("REDIS_URL", "redis://127.0.0.1:6379");
        set_env_if_unset!("AXUM_WEBAUTHN_RP_ID", "localhost");
        set_env_if_unset!("AXUM_WEBAUTHN_ORIGIN", "http://localhost:8080");
        set_env_if_unset!("AXUM_WEBAUTHN_RP_NAME", "Test App");
        set_env_if_unset!("AXUM_METRICS_TYPE", "noop");
    });

    // Database init OUTSIDE call_once (but it's idempotent anyway)
    // Run async DB init on the shared runtime
    let _ = init_database_with_retry_from_env().await;
}

pub struct TestServer {
    pub addr: std::net::SocketAddr,
    pub client: Client,
}

impl TestServer {
    // ---
    pub async fn new() -> Self {
        // --

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
