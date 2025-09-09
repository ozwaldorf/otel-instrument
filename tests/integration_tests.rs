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

// Test struct destructuring patterns
#[instrument]
fn instrumented_struct_destructure(_Test { id }: _Test) -> Result<String> {
    Ok(format!("Processing id: {}", id))
}

// Test tuple struct destructuring patterns  
struct StateWrapper(String);

#[instrument]
fn instrumented_tuple_struct_destructure(StateWrapper(state): StateWrapper) -> Result<String> {
    Ok(format!("State: {}", state))
}

struct _Test {
    id: u32,
}

impl _Test {
    #[instrument]
    async fn test_function(self, param: &str) -> Result<String> {
        Ok(format!("Hello, {param}"))
    }

    #[instrument]
    fn sync_test_function(self, param: &str) -> Result<String> {
        Ok(format!("Hello, {param}"))
    }

    // Test self token in fields
    #[instrument(fields(struct_id = self.id, param_value = param))]
    async fn test_self_in_fields(self, param: &str) -> Result<String> {
        Ok(format!("Hello, {param} from id {}", self.id))
    }

    #[instrument(fields(struct_id = self.id, param_value = param))]
    fn sync_test_self_in_fields(self, param: &str) -> Result<String> {
        Ok(format!("Hello, {param} from id {}", self.id))
    }

    // Test shorthand field syntax (name without =) - using variables in scope
    #[instrument(fields(param))]
    async fn test_shorthand_fields(self, param: &str) -> Result<String> {
        let id = self.id;
        Ok(format!("Hello, {param} with id {id}"))
    }

    #[instrument(fields(param))]
    fn sync_test_shorthand_fields(self, param: &str) -> Result<String> {
        let id = self.id;
        Ok(format!("Hello, {param} with id {id}"))
    }

    // Test mixed shorthand and explicit field syntax
    #[instrument(fields(param, custom_field = "custom_value"))]
    async fn test_mixed_fields(self, param: &str) -> Result<String> {
        let id = self.id * 2;
        Ok(format!("Mixed: {param} with doubled id {id}"))
    }
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

// Test shorthand fields functionality for regular functions
#[instrument(fields(param, user_count))]
async fn test_shorthand_fields_function(param: &str, user_count: i32) -> Result<String> {
    Ok(format!("Hello, {param}, count: {user_count}"))
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

#[tokio::test]
async fn test_self_in_fields_async() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let test_instance = _Test { id: 42 };
    let result = test_instance.test_self_in_fields("world").await;
    assert_eq!(result.unwrap(), "Hello, world from id 42");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_self_in_fields_sync() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let test_instance = _Test { id: 123 };
    let result = test_instance.sync_test_self_in_fields("universe");
    assert_eq!(result.unwrap(), "Hello, universe from id 123");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_shorthand_fields_async() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let test_instance = _Test { id: 456 };
    let result = test_instance.test_shorthand_fields("test").await;
    assert_eq!(result.unwrap(), "Hello, test with id 456");
    tracer_provider.shutdown().unwrap();
}

#[test]
fn test_shorthand_fields_sync() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let test_instance = _Test { id: 789 };
    let result = test_instance.sync_test_shorthand_fields("sync");
    assert_eq!(result.unwrap(), "Hello, sync with id 789");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_mixed_fields() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let test_instance = _Test { id: 100 };
    let result = test_instance.test_mixed_fields("mixed").await;
    assert_eq!(result.unwrap(), "Mixed: mixed with doubled id 200");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_shorthand_fields_function_test() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let result = test_shorthand_fields_function("world", 42).await;
    assert_eq!(result.unwrap(), "Hello, world, count: 42");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_struct_destructuring() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let test_instance = _Test { id: 42 };
    let result = instrumented_struct_destructure(test_instance);
    assert_eq!(result.unwrap(), "Processing id: 42");
    tracer_provider.shutdown().unwrap();
}

#[tokio::test]
async fn test_tuple_struct_destructuring() {
    let tracer_provider = setup_otlp_tracer().unwrap();
    let state_wrapper = StateWrapper("app_state".to_string());
    let result = instrumented_tuple_struct_destructure(state_wrapper);
    assert_eq!(result.unwrap(), "State: app_state");
    tracer_provider.shutdown().unwrap();
}
