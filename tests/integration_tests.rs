use eyre::{Result, bail};
use opentelemetry::global;
use opentelemetry::trace::TraceContextExt;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use otel_instrument::{instrument, tracer_name};

tracer_name!("otel-instrument-tests");

// Utility function to setup OpenTelemetry OTLP trace exporter with HTTP
fn setup_otlp_tracer() -> Result<SdkTracerProvider> {
    let otlp_exporter = SpanExporter::builder().with_http().build()?;
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_id_generator(RandomIdGenerator::default())
        .with_sampler(Sampler::AlwaysOn)
        .build();
    global::set_tracer_provider(tracer_provider.clone());
    Ok(tracer_provider)
}

// Basic functionality tests
#[instrument]
async fn test_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

#[instrument]
async fn failing_function() -> Result<()> {
    bail!("Test error")
}

// Test skip functionality
#[instrument(skip(password))]
async fn test_skip_function(username: &str, _password: &str) -> Result<String> {
    Ok(format!("Hello, {username}"))
}

// Test skip_all functionality
#[instrument(skip_all)]
async fn test_skip_all_function(_secret: &str, token: &str) -> Result<String> {
    Ok(format!("Success: {token}"))
}

// Test fields functionality
#[instrument(fields(custom_field = "custom_value", user_id = 123))]
async fn test_fields_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

// Test ret functionality
#[instrument(ret)]
async fn test_ret_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

// Test err functionality
#[instrument(err = e.as_ref())]
async fn test_err_function() -> Result<()> {
    bail!("Test error")
}

// Test err functionality with eyre support
#[instrument(err = e.as_ref())]
async fn test_err_eyre_function() -> Result<()> {
    bail!("Test eyre error")
}

// Test name functionality
#[instrument(name = "custom_span_name")]
async fn test_name_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

// Test name with empty string (should use function name as fallback)
#[instrument(name = "")]
async fn test_empty_name_function() -> Result<String> {
    Ok("test".to_string())
}

// Test name combined with other attributes
#[instrument(name = "login_operation", skip(password), ret)]
async fn test_name_with_other_attrs(username: &str, _password: &str) -> Result<String> {
    Ok(format!("User: {username}"))
}

// Test combination of features
#[instrument(skip(password), ret, err = e.as_ref(), fields(operation = "login"))]
async fn test_combined_function(username: &str, _password: &str) -> Result<String> {
    if username == "admin" {
        Ok(format!("Welcome, {username}"))
    } else {
        bail!("Access denied")
    }
}

// Test parent attribute with Context
#[instrument(parent = _parent_ctx)]
async fn test_parent_context_function(
    param: &str,
    _parent_ctx: opentelemetry::Context,
) -> Result<String> {
    Ok(format!("Child span with param: {param}"))
}

// Test parent attribute with expression
#[instrument(parent = get_parent_context())]
async fn test_parent_expression_function(param: &str) -> Result<String> {
    Ok(format!("Child span with param: {param}"))
}

// Test parent combined with other attributes
#[instrument(parent = _parent_ctx, name = "child_operation", ret)]
async fn test_parent_with_other_attrs(
    _parent_ctx: opentelemetry::Context,
    value: i32,
) -> Result<i32> {
    Ok(value * 2)
}

// Helper function to create a parent context
fn get_parent_context() -> opentelemetry::Context {
    use opentelemetry::{global, trace::Tracer};
    let tracer = global::tracer("test-tracer");
    let span = tracer.start("parent-span");
    opentelemetry::Context::current_with_span(span)
}

// Sync function tests
#[instrument]
fn sync_test_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

#[instrument]
fn sync_failing_function() -> Result<()> {
    bail!("Test error")
}

#[instrument(skip(password))]
fn sync_test_skip_function(username: &str, _password: &str) -> Result<String> {
    Ok(format!("Hello, {username}"))
}

#[instrument(skip_all)]
fn sync_test_skip_all_function(_secret: &str, token: &str) -> Result<String> {
    Ok(format!("Success: {token}"))
}

#[instrument(fields(custom_field = "custom_value", user_id = 123))]
fn sync_test_fields_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

#[instrument(ret)]
fn sync_test_ret_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

#[instrument(err = e.as_ref())]
fn sync_test_err_function() -> Result<()> {
    bail!("Test error")
}

#[instrument(name = "sync_custom_span_name")]
fn sync_test_name_function(param: &str) -> Result<String> {
    Ok(format!("Hello, {param}"))
}

#[instrument(skip(password), ret, err = e.as_ref(), fields(operation = "sync_login"))]
fn sync_test_combined_function(username: &str, _password: &str) -> Result<String> {
    if username == "admin" {
        Ok(format!("Welcome, {username}"))
    } else {
        bail!("Access denied")
    }
}

#[instrument(parent = _parent_ctx)]
fn sync_test_parent_context_function(
    param: &str,
    _parent_ctx: opentelemetry::Context,
) -> Result<String> {
    Ok(format!("Child span with param: {param}"))
}

#[tokio::test]
async fn test_successful_instrumentation() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_error_instrumentation() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = failing_function().await;
    assert!(result.is_err());
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_skip_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_skip_function("admin", "secret123").await;
    assert_eq!(result.unwrap(), "Hello, admin");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_skip_all_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_skip_all_function("secret", "token").await;
    assert_eq!(result.unwrap(), "Success: token");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_fields_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_fields_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_ret_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_ret_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_err_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_err_function().await;
    assert!(result.is_err());
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_name_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_name_function("world").await;
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_empty_name_fallback() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_empty_name_function().await;
    assert_eq!(result.unwrap(), "test");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_name_with_other_attributes() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_name_with_other_attrs("admin", "secret").await;
    assert_eq!(result.unwrap(), "User: admin");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_combined_attributes_success() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_combined_function("admin", "password123").await;
    assert_eq!(result.unwrap(), "Welcome, admin");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_combined_attributes_failure() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_combined_function("user", "password123").await;
    assert!(result.is_err());
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_parent_context_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    use opentelemetry::{global, trace::Tracer};
    let tracer = global::tracer("test-tracer");
    let parent_span = tracer.start("parent-span");
    let parent_ctx = opentelemetry::Context::current_with_span(parent_span);

    let result = test_parent_context_function("test", parent_ctx).await;
    assert_eq!(result.unwrap(), "Child span with param: test");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_parent_expression_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_parent_expression_function("test").await;
    assert_eq!(result.unwrap(), "Child span with param: test");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_parent_with_other_attributes() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    use opentelemetry::{global, trace::Tracer};
    let tracer = global::tracer("test-tracer");
    let parent_span = tracer.start("parent-span");
    let parent_ctx = opentelemetry::Context::current_with_span(parent_span);

    let result = test_parent_with_other_attrs(parent_ctx, 21).await;
    assert_eq!(result.unwrap(), 42);
    tracer_provider.shutdown().unwrap();
}

// Sync function tests
#[test]
fn test_sync_successful_instrumentation() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_function("world");
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_error_instrumentation() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_failing_function();
    assert!(result.is_err());
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_skip_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_skip_function("admin", "secret123");
    assert_eq!(result.unwrap(), "Hello, admin");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_skip_all_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_skip_all_function("secret", "token");
    assert_eq!(result.unwrap(), "Success: token");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_fields_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_fields_function("world");
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_ret_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_ret_function("world");
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_err_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_err_function();
    assert!(result.is_err());
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_name_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_name_function("world");
    assert_eq!(result.unwrap(), "Hello, world");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_combined_attributes_success() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_combined_function("admin", "password123");
    assert_eq!(result.unwrap(), "Welcome, admin");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_combined_attributes_failure() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = sync_test_combined_function("user", "password123");
    assert!(result.is_err());
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_sync_parent_context_attribute() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    use opentelemetry::{global, trace::Tracer};
    let tracer = global::tracer("test-tracer");
    let parent_span = tracer.start("parent-span");
    let parent_ctx = opentelemetry::Context::current_with_span(parent_span);

    let result = sync_test_parent_context_function("test", parent_ctx);
    assert_eq!(result.unwrap(), "Child span with param: test");
    tracer_provider.shutdown().unwrap();
}
