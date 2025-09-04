# otel-instrument

A procedural macro for instrumenting async Rust functions with OpenTelemetry tracing. Similar to the `tracing` crate's `#[instrument]` macro but specifically designed for OpenTelemetry spans.

## Features

- **Async-focused**: Designed specifically for async functions
- **Parameter capture**: Automatically records function parameters as span attributes
- **Custom fields**: Add custom attributes to spans
- **Return value capture**: Optionally record return values
- **Error handling**: Enhanced error capture with span status updates
- **Flexible skipping**: Skip specific parameters or all parameters from instrumentation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
otel-instrument = "0.1.0"
```

## Usage

### Basic Usage

```rust
use otel_instrument::{instrument, tracer_name};

// Define a tracer name for the instrument macros. Must be in module scope
tracer_name!("my-service");

#[instrument]
async fn my_function(user_id: u64, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Your async code here
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
#[derive(Debug)]
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

#[derive(Debug)]
enum MathError {
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

### Combined Usage

Use multiple attributes together:

```rust
use otel_instrument::{instrument, tracer_name};

tracer_name!("auth-service");

#[derive(Debug)]
struct AuthToken { token: String }
#[derive(Debug)]
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

## Requirements

- Functions must be `async`
- OpenTelemetry must be properly configured in your application
- The macro uses the global tracer named "otel-instrument"
