# otel-instrument

A procedural macro for instrumenting Rust functions with OpenTelemetry tracing. Similar to the `tracing` crate's `#[instrument]` macro but specifically designed for OpenTelemetry spans. Supports both async and sync functions.

## Features

- **Async and sync support**: Works with both async and synchronous functions
- **Parameter capture**: Automatically records function parameters as span attributes
- **Custom fields**: Add custom attributes to spans
- **Return value capture**: Optionally record return values
- **Error handling**: Enhanced error capture with span status updates
- **Flexible skipping**: Skip specific parameters or all parameters from instrumentation
- **Parent span support**: Set explicit parent-child span relationships

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
otel-instrument = "0.1.0"
```

## Usage

### Basic Usage

#### Async Functions

```rust
use otel_instrument::{instrument, tracer_name};

// Define a tracer name for the instrument macros. Must be in module scope
tracer_name!("my-service");

#[instrument]
async fn my_async_function(user_id: u64, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Your async code here
    Ok(format!("Hello, {}", name))
}
```

#### Sync Functions

```rust
use otel_instrument::{instrument, tracer_name};

// Define a tracer name for the instrument macros. Must be in module scope
tracer_name!("my-service");

#[instrument]
fn my_sync_function(user_id: u64, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Your sync code here
    Ok(format!("Hello, {}", name))
}
```

### Skip Parameters

Skip specific parameters from being recorded as span attributes:

```rust
use otel_instrument::{instrument, tracer_name};

tracer_name!("auth-service");

#[derive(Debug)]
struct User { name: String }

#[derive(thiserror::Error, Debug)]
#[error("failed to authenticate")]
struct AuthError;

#[instrument(skip(password))]
async fn login(username: &str, password: &str) -> Result<User, AuthError> {
    // password won't be recorded as a span attribute
    Ok(User { name: username.to_string() })
}
```

Skip all parameters:

```rust
use otel_instrument::{instrument, tracer_name};

tracer_name!("secure-service");

#[instrument(skip_all)]
async fn sensitive_operation(secret: &str) -> Result<(), Box<dyn std::error::Error>> {
    // No parameters will be recorded
    Ok(())
}
```

### Custom Fields

Add custom fields to your spans:

```rust
use otel_instrument::{instrument, tracer_name};

tracer_name!("database-service");

#[derive(Debug)]
struct User { id: u64 }

#[instrument(fields(operation = "user_lookup", component = "auth"))]
async fn find_user(id: u64) -> Result<User, Box<dyn std::error::Error>> {
    Ok(User { id })
}
```

### Return Value and Error Capture

Record return values and enhanced error information:

```rust
use otel_instrument::{instrument, tracer_name};

tracer_name!("math-service");

#[derive(thiserror::Error, Debug)]
enum MathError {
    #[error("division by zero")]
    DivisionByZero,
}

#[instrument(ret, err)]
async fn calculate(a: i32, b: i32) -> Result<i32, MathError> {
    if b == 0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}
```

### Parent Span Context

Set a parent context to create child spans:

```rust
use otel_instrument::{instrument, tracer_name};
use opentelemetry::{global, trace::{Tracer, TraceContextExt}, Context};

tracer_name!("parent-child-service");

#[derive(Debug)]
struct UserData { id: u64 }

// Using a context parameter
#[instrument(parent = parent_ctx)]
async fn child_operation(
    user_id: u64,
    parent_ctx: opentelemetry::Context
) -> Result<UserData, Box<dyn std::error::Error>> {
    Ok(UserData { id: user_id })
}

// Using an expression
#[instrument(parent = get_parent_context())]
async fn another_child_operation(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(data.to_uppercase())
}

fn get_parent_context() -> opentelemetry::Context {
    let tracer = global::tracer("parent-tracer");
    let span = tracer.start("parent-span");
    opentelemetry::Context::current_with_span(span)
}
```

### Combined Usage

Use multiple attributes together:

```rust
use otel_instrument::{instrument, tracer_name};

tracer_name!("auth-service");

#[derive(Debug)]
struct AuthToken { token: String }

#[derive(thiserror::Error, Debug)]
#[error("failed to authenticate")]
struct AuthError;

#[instrument(
    skip(password),
    fields(service = "auth", version = "1.0"),
    ret,
    err
)]
async fn authenticate_user(
    username: &str,
    password: &str,
    ip_address: &str,
) -> Result<AuthToken, AuthError> {
    Ok(AuthToken { token: "token123".to_string() })
}

// Combined with parent
#[instrument(
    parent = parent_ctx,
    name = "user_validation",
    skip(password),
    ret
)]
async fn validate_user(
    parent_ctx: opentelemetry::Context,
    username: &str,
    password: &str,
) -> Result<bool, AuthError> {
    Ok(username == "admin")
}
```

### Sync Function Examples

The macro works identically with synchronous functions:

```rust
use otel_instrument::{instrument, tracer_name};

tracer_name!("sync-service");

#[derive(thiserror::Error, Debug)]
#[error("failed to process")]
struct ProcessingError;

// Basic sync function
#[instrument]
fn process_data(input: &str) -> Result<String, ProcessingError> {
    Ok(input.to_uppercase())
}

// Sync function with custom attributes
#[instrument(
    skip(secret_key),
    fields(operation = "encryption", version = "1.0"),
    ret,
    err
)]
fn encrypt_data(data: &str, secret_key: &str) -> Result<String, ProcessingError> {
    // Encryption logic here
    Ok(format!("encrypted_{}", data))
}

// Sync function with parent context
#[instrument(parent = parent_ctx)]
fn child_processing(
    parent_ctx: opentelemetry::Context,
    data: &str,
) -> Result<String, ProcessingError> {
    Ok(format!("processed: {}", data))
}
```

## Attributes

### `skip(param1, param2, ...)`
Skip specific function parameters from being recorded as span attributes.

### `skip_all`
Skip all function parameters from being recorded as span attributes.

### `fields(key = value, ...)`
Add custom fields/attributes to the span. Values are evaluated and formatted using `Debug`.

### `ret`
Record the return value as a span attribute named "return".

### `err`
Record error values as span attributes and set appropriate span status. When an error occurs, the span status is set to error with the error description.

### `parent = <expression>`
Set a parent context for the span. The expression must evaluate to something that implements `Into<opentelemetry::Context>`. This allows creating child spans with explicit parent-child relationships.

## Requirements

- Functions can be either `async` or synchronous
- OpenTelemetry must be properly configured in your application
- The macro uses the global tracer specified by the `tracer_name!` macro
