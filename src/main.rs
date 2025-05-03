use anyhow::Result;
use axum_quickstart::create_app; // <-- this will now work
use futures::FutureExt;
use std::env;
use tracing::{info, Level};

use tracing_subscriber::fmt::format::FmtSpan;

// Initialize tracing subscriber
fn init_tracing() {
    // ---
    let span_events = match env::var("AXUM_SPAN_EVENTS").as_deref() {
        Ok("full") => FmtSpan::FULL, // ENTER, EXIT, CLOSE with timing
        Ok("enter_exit") => FmtSpan::ENTER | FmtSpan::EXIT, // Only ENTER and EXIT
        _ => FmtSpan::CLOSE,         // Default: only CLOSE timing
    };

    // Determine log level from env, default to DEBUG
    let level = match env::var("AXUM_LOG_LEVEL").ok().as_deref() {
        Some("trace") => Level::TRACE,
        Some("debug") => Level::DEBUG,
        Some("info") => Level::INFO,
        Some("warn") => Level::WARN,
        Some("error") => Level::ERROR,
        _ => Level::DEBUG, // Default
    };

    tracing_subscriber::fmt()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(span_events)
        .with_max_level(level)
        .compact()
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // ---
    // Initialize tracing subscriber to log to stdout
    init_tracing();

    tracing::info!("Starting axum server...");

    let app = create_app()?;

    // Get optional bind endpoint from environment
    let endpoint = env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    info!("Starting at endpoint:{}", endpoint);
    info!(
        "Starting Axum Quick-Start API server v{}...",
        env!("CARGO_PKG_VERSION")
    );

    let listener = tokio::net::TcpListener::bind(&endpoint).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(real_shutdown_signal())
        .await?;

    Ok(())
}

fn real_shutdown_signal() -> impl std::future::Future<Output = ()> {
    use futures::future;
    use tokio::signal::ctrl_c;
    use tokio::signal::unix::{signal, SignalKind};

    let ctrl_c = async {
        ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    let sigterm = async {
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    future::select(Box::pin(ctrl_c), Box::pin(sigterm)).map(|_| ())
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn shutdown_signal_completes() {
        let fake_signal = async {};
        shutdown_signal(fake_signal).await;
        // Optional: assert logs, state flags, etc.
    }

    async fn shutdown_signal<F>(signal: F)
    where
        F: std::future::Future<Output = ()>,
    {
        signal.await;
        tracing::info!("Shutdown signal received. Closing server gracefully...");
    }
}
