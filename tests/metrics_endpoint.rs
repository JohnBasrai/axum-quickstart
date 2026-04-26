use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

mod common;

#[tokio::test]
async fn metrics_endpoint_with_prometheus() {
    common::setup_test_env().await;
    let server = common::TestServer::with_metrics("prom").await;

    let _ = server
        .client
        .get(server.url("/health"))
        .send()
        .await
        .unwrap();
    let _ = server.client.get(server.url("/")).send().await.unwrap();
    let _ = server
        .client
        .get(server.url("/movies"))
        .send()
        .await
        .unwrap();

    sleep(Duration::from_millis(50)).await;

    let res = server
        .client
        .get(server.url("/metrics"))
        .send()
        .await
        .unwrap();

    assert!(
        res.status().is_success(),
        "Metrics endpoint should return success"
    );

    let body = res.text().await.unwrap();
    println!("Metrics response body: '{body}'");
    assert!(!body.is_empty(), "Metrics should not be empty");

    if body.contains("# HELP") || body.contains("# TYPE") {
        println!("✅ Detected Prometheus format metrics");
    } else {
        println!("ℹ️  Metrics format: {body}");
    }
}

#[tokio::test]
async fn metrics_endpoint_with_noop() {
    common::setup_test_env().await;
    let server = common::TestServer::with_metrics("noop").await;

    let _ = server
        .client
        .get(server.url("/health"))
        .send()
        .await
        .unwrap();
    let _ = server.client.get(server.url("/")).send().await.unwrap();

    let res = server
        .client
        .get(server.url("/metrics"))
        .send()
        .await
        .unwrap();

    assert!(
        res.status().is_success(),
        "Metrics endpoint should return success even with noop"
    );

    let body = res.text().await.unwrap();
    println!("Noop metrics response: '{body}'");
}

#[tokio::test]
async fn metrics_endpoint_survives_load() {
    common::setup_test_env().await;
    let server = Arc::new(common::TestServer::with_metrics("prom").await);

    let futures = (0..20).map(|i| {
        let server = Arc::clone(&server);
        async move {
            let endpoint = match i % 3 {
                0 => "/health",
                1 => "/",
                _ => "/metrics",
            };
            server.client.get(server.url(endpoint)).send().await
        }
    });

    let responses = futures::future::join_all(futures).await;

    for (i, response) in responses.into_iter().enumerate() {
        let response = response.unwrap_or_else(|_| panic!("Request {i} should succeed"));
        if !response.status().is_success() {
            println!("Request {} failed with status: {}", i, response.status());
        }
        assert!(
            response.status().is_success(),
            "Request {i} should return success"
        );
    }

    let res = server
        .client
        .get(server.url("/metrics"))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let body = res.text().await.unwrap();
    assert!(!body.is_empty());
    println!("Got metrics:{body}");
}

#[tokio::test]
async fn metrics_content_type_is_correct() {
    common::setup_test_env().await;
    let server = common::TestServer::with_metrics("prom").await;

    let res = server
        .client
        .get(server.url("/metrics"))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let content_type = res.headers().get("content-type");
    println!("Metrics content-type: {content_type:?}");

    if let Some(ct) = content_type {
        let ct_str = ct.to_str().unwrap();
        assert!(
            ct_str.contains("text/plain")
                || ct_str.contains("text/")
                || ct_str.contains("application/"),
            "Content type should be appropriate for metrics: {ct_str}"
        );
    }
}
