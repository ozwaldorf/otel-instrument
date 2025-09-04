use opentelemetry::trace::TraceContextExt;
use otel_instrument::{instrument, tracer_name};

tracer_name!("asdf");

// Basic functionality tests
#[instrument]
async fn test_function(param: &str) -> Result<String, String> {
    Ok(format!("Hello, {param}"))
}

#[instrument]
async fn failing_function() -> Result<(), String> {
    Err("Test error".to_string())
}

// Test skip functionality
#[instrument(skip(password))]
async fn test_skip_function(username: &str, _password: &str) -> Result<String, String> {
    Ok(format!("Hello, {username}"))
}

// Test skip_all functionality
#[instrument(skip_all)]
async fn test_skip_all_function(_secret: &str, token: &str) -> Result<String, String> {
    Ok(format!("Success: {token}"))
}

// Test fields functionality
#[instrument(fields(custom_field = "custom_value", user_id = 123))]
async fn test_fields_function(param: &str) -> Result<String, String> {
    Ok(format!("Hello, {param}"))
}

// Test ret functionality
#[instrument(ret)]
async fn test_ret_function(param: &str) -> Result<String, String> {
    Ok(format!("Hello, {param}"))
}

// Test err functionality
#[instrument(err)]
async fn test_err_function() -> Result<(), String> {
    Err("Test error".to_string())
}

// Test name functionality
#[instrument(name = "custom_span_name")]
async fn test_name_function(param: &str) -> Result<String, String> {
    Ok(format!("Hello, {param}"))
}

// Test name with empty string (should use function name as fallback)
#[instrument(name = "")]
async fn test_empty_name_function() -> Result<String, String> {
    Ok("test".to_string())
}

// Test name combined with other attributes
#[instrument(name = "login_operation", skip(password), ret)]
async fn test_name_with_other_attrs(username: &str, _password: &str) -> Result<String, String> {
    Ok(format!("User: {username}"))
}

// Test combination of features
#[instrument(skip(password), ret, err, fields(operation = "login"))]
async fn test_combined_function(username: &str, _password: &str) -> Result<String, String> {
    if username == "admin" {
        Ok(format!("Welcome, {username}"))
    } else {
        Err("Access denied".to_string())
    }
}

// Test parent attribute with Context
#[instrument(parent = parent_ctx)]
async fn test_parent_context_function(
    param: &str,
    parent_ctx: opentelemetry::Context,
) -> Result<String, String> {
    Ok(format!("Child span with param: {param}"))
}

// Test parent attribute with expression
#[instrument(parent = get_parent_context())]
async fn test_parent_expression_function(param: &str) -> Result<String, String> {
    Ok(format!("Child span with param: {param}"))
}

// Test parent combined with other attributes
#[instrument(parent = parent_ctx, name = "child_operation", ret)]
async fn test_parent_with_other_attrs(
    parent_ctx: opentelemetry::Context,
    value: i32,
) -> Result<i32, String> {
    Ok(value * 2)
}

// Helper function to create a parent context
fn get_parent_context() -> opentelemetry::Context {
    use opentelemetry::{global, trace::Tracer};
    let tracer = global::tracer("test-tracer");
    let span = tracer.start("parent-span");
    opentelemetry::Context::current_with_span(span)
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

#[tokio::test]
async fn test_skip_attribute() {
    let result = test_skip_function("admin", "secret123").await;
    assert_eq!(result.unwrap(), "Hello, admin");
}

#[tokio::test]
async fn test_skip_all_attribute() {
    let result = test_skip_all_function("secret", "token").await;
    assert_eq!(result.unwrap(), "Success: token");
}

#[tokio::test]
async fn test_fields_attribute() {
    let result = test_fields_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
}

#[tokio::test]
async fn test_ret_attribute() {
    let result = test_ret_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
}

#[tokio::test]
async fn test_err_attribute() {
    let result = test_err_function().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_name_attribute() {
    let result = test_name_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
}

#[tokio::test]
async fn test_empty_name_fallback() {
    let result = test_empty_name_function().await;
    assert_eq!(result.unwrap(), "test");
}

#[tokio::test]
async fn test_name_with_other_attributes() {
    let result = test_name_with_other_attrs("admin", "secret").await;
    assert_eq!(result.unwrap(), "User: admin");
}

#[tokio::test]
async fn test_combined_attributes_success() {
    let result = test_combined_function("admin", "password123").await;
    assert_eq!(result.unwrap(), "Welcome, admin");
}

#[tokio::test]
async fn test_combined_attributes_failure() {
    let result = test_combined_function("user", "password123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parent_context_attribute() {
    use opentelemetry::{global, trace::Tracer};
    let tracer = global::tracer("test-tracer");
    let parent_span = tracer.start("parent-span");
    let parent_ctx = opentelemetry::Context::current_with_span(parent_span);

    let result = test_parent_context_function("test", parent_ctx).await;
    assert_eq!(result.unwrap(), "Child span with param: test");
}

#[tokio::test]
async fn test_parent_expression_attribute() {
    let result = test_parent_expression_function("test").await;
    assert_eq!(result.unwrap(), "Child span with param: test");
}

#[tokio::test]
async fn test_parent_with_other_attributes() {
    use opentelemetry::{global, trace::Tracer};
    let tracer = global::tracer("test-tracer");
    let parent_span = tracer.start("parent-span");
    let parent_ctx = opentelemetry::Context::current_with_span(parent_span);

    let result = test_parent_with_other_attrs(parent_ctx, 21).await;
    assert_eq!(result.unwrap(), 42);
}
