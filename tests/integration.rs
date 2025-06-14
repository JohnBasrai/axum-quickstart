use axum_quickstart::create_router;

// Add other integration tests here as needed
#[tokio::test]
async fn basic_integration_test() {
    // Test that the router can be created successfully
    let _router = create_router().expect("Should be able to create router");
}
