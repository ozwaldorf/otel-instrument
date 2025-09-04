use otel_instrument::instrument;

#[instrument]
async fn test_function(param: &str) -> Result<String, String> {
    Ok(format!("Hello, {param}"))
}

#[instrument]
async fn failing_function() -> Result<(), String> {
    Err("Test error".to_string())
}

#[tokio::test]
async fn test_successful_instrumentation() {
    let result = test_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
}

#[tokio::test]
async fn test_error_instrumentation() {
    let result = failing_function().await;
    assert!(result.is_err());
}

