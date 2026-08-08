---
title: Logger Reference
description: Structured logging configuration and API reference.
sidebar:
  order: 4
---

# Logger (Structured Logging)

:::tip[Quick Start]
Enable structured logging with one line in your middleware chain:

```rust
use rusta_logger::logger_middleware;
use rusta::MiddlewareChain;

let middleware = MiddlewareChain::new().chain(logger_middleware);
```

:::

## Configuration

```rust
use rusta_logger::{Logger, config, LogLevel};

let logger = Logger::configure(
    config()
        .service_name("my-service")           // Required
        .service_version("1.0.0")             // Optional
        .environment("production")              // Optional
        .min_level(LogLevel::Info)              // Optional, default: Info
        .add_classification("PUBLIC", "public.ndjson")
        .add_classification("CONFIDENTIAL", "confidential.ndjson")
        .default_classification("PUBLIC")     // Optional, default: "PUBLIC"
        .correlation_id_header("X-Correlation-ID") // Optional
        .build(),
).await;
```

## LoggerConfigBuilder Methods

| Method                           | Description                                         |
| -------------------------------- | --------------------------------------------------- |
| `service_name(name)`             | Set service identifier (required)                   |
| `service_version(version)`       | Set service version                                 |
| `environment(env)`               | Set environment (e.g., "production", "development") |
| `server_name(name)`              | Set server name                                     |
| `min_level(level)`               | Set minimum log level                               |
| `add_classification(name, path)` | Add log file for classification                     |
| `default_classification(name)`   | Set default classification                          |
| `correlation_id_header(header)`  | Set header for correlation ID propagation           |
| `adapter(adapter)`               | Set custom log adapter                              |
| `build()`                        | Build configuration                                 |

## Log Levels

```rust
pub enum LogLevel {
    Trace = 0,  // Most verbose
    Debug,
    Info,
    Warn,
    Error,      // Least verbose
}
```

## Logging Methods

### Basic Logging

```rust
Logger::trace("Detailed trace", None);
Logger::debug("Debug info", None);
Logger::info("Info message", None);
Logger::warn("Warning", None);
Logger::error("Error occurred", None);
```

### With Context

```rust
use rusta_logger::LogOptions;
use serde_json::json;

Logger::info(
    "User created",
    Some(LogOptions {
        context: Some([
            ("user_id", json!(user.id)),
            ("email", json!(user.email)),
        ].into()),
        ..Default::default()
    })
);
```

### With Classification

```rust
Logger::log(
    LogLevel::Info,
    "Sensitive operation",
    Some(LogOptions {
        classification: Some("CONFIDENTIAL".to_string()),
        ..Default::default()
    })
);
```

## LogEntry Structure

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "INFO",
  "message": "User logged in",
  "classification": "PUBLIC",
  "correlation_id": "optional-correlation-id",
  "service": {
    "service_name": "my-service",
    "service_version": "1.0.0",
    "environment": "production"
  },
  "context": {
    "user_id": "123",
    "ip_address": "192.168.1.1"
  }
}
```

## LogOptions Fields

| Field            | Type               | Description                     |
| ---------------- | ------------------ | ------------------------------- |
| `classification` | `Option<String>`   | Override default classification |
| `context`        | `Option<Metadata>` | Additional structured fields    |

## Middleware Integration

```rust
use rusta_logger::logger_middleware;
use rusta::{App, MiddlewareChain};

let middleware = MiddlewareChain::new()
    .chain(logger_middleware);

App::new()
    .container(container)
    .middleware(middleware)
    .run("0.0.0.0:3000", |res| match res {
        Ok(addr) => println!("Listening on {}", addr),
        Err(msg) => eprintln!("Startup error: {}", msg),
    })
    .await;
```

The middleware automatically:

- Extracts/propagates correlation IDs
- Sets up task-local context for all log calls
- Injects correlation ID into response headers

## Custom Adapters

Implement `LogAdapter` for custom formatting:

```rust
use rusta_logger::{LogAdapter, LogEntry};

pub struct JsonAdapter;

impl LogAdapter for JsonAdapter {
    fn format(&self, entry: &LogEntry) -> String {
        serde_json::to_string(entry).unwrap() + "\n"
    }
}

// Use in configuration
let logger = Logger::configure(
    config()
        .adapter(Box::new(JsonAdapter))
        .build(),
).await;
```

## Shutdown

Gracefully shutdown the logger (drains pending entries):

```rust
Logger::shutdown().await;
```
