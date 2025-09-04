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

// Test combination of features
#[instrument(skip(password), ret, err, fields(operation = "login"))]
async fn test_combined_function(username: &str, _password: &str) -> Result<String, String> {
    if username == "admin" {
        Ok(format!("Welcome, {username}"))
    } else {
        Err("Access denied".to_string())
    }
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
async fn test_combined_attributes_success() {
    let result = test_combined_function("admin", "password123").await;
    assert_eq!(result.unwrap(), "Welcome, admin");
}

#[tokio::test]
async fn test_combined_attributes_failure() {
    let result = test_combined_function("user", "password123").await;
    assert!(result.is_err());
}
