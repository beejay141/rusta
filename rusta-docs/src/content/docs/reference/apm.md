---
title: APM Reference
description: Application Performance Monitoring configuration and API reference.
sidebar:
  order: 3
---

# APM (Application Performance Monitoring)

:::tip[Quick Start]
Enable APM with one line in your middleware chain:

```rust
use rusta_apm::apm_middleware;
use rusta::MiddlewareChain;

let middleware = MiddlewareChain::new().chain(apm_middleware);
```

:::

## Configuration

```rust
use rusta_apm::{Apm, config};

let apm = Apm::configure(
    config()
        .service_name("my-service")           // Required
        .service_version("1.0.0")             // Optional
        .environment("production")              // Optional
        .log_path("apm.ndjson")               // Optional, default: "apm.ndjson"
        .correlation_id_header("X-Correlation-ID") // Optional
        .build(),
).await;
```

## ApmConfigBuilder Methods

| Method                          | Description                                         |
| ------------------------------- | --------------------------------------------------- |
| `service_name(name)`            | Set service identifier (required)                   |
| `service_version(version)`      | Set service version                                 |
| `environment(env)`              | Set environment (e.g., "production", "development") |
| `server_name(name)`             | Set server name                                     |
| `log_path(path)`                | Set NDJSON output path                              |
| `correlation_id_header(header)` | Set header for correlation ID propagation           |
| `adapter(adapter)`              | Set custom log adapter                              |
| `build()`                       | Build configuration                                 |

## Transaction API

### start_transaction

Create a new transaction:

```rust
let handle = Apm::start_transaction("GET /users", "request", Some(metadata));
// ... work ...
handle.end(Some("HTTP 200"), Some(metadata));
```

### wrap_transaction

Execute future within transaction scope:

```rust
let result = Apm::wrap_transaction(
    "database.query",
    "db",
    Some(metadata),
    || async { db.query().await }
).await;
```

### wrap_transaction_future

Same as wrap_transaction but takes future directly:

```rust
let result = Apm::wrap_transaction_future(
    "external.api",
    "http",
    None,
    reqwest::get("https://api.example.com").await
).await;
```

## Span API

### start_span

Create a span within active transaction:

```rust
let span = Apm::start_span("cache.lookup", "cache", Some(metadata));
// ... work ...
span.end(Some(metadata));
```

### wrap_span

Execute future within span:

```rust
let result = Apm::wrap_span(
    "cache.lookup",
    "cache",
    Some(metadata),
    || async { cache.get(key).await }
).await;
```

### wrap_span_future

Same as wrap_span but takes future directly:

```rust
let result = Apm::wrap_span_future(
    "cache.lookup",
    "cache",
    None,
    cache.get(key)
).await;
```

## TransactionHandle Methods

| Method                  | Description                                        |
| ----------------------- | -------------------------------------------------- |
| `active_txn()`          | Get Arc<ActiveTransaction> for context propagation |
| `end(result, metadata)` | End transaction with result and optional metadata  |

## SpanHandle Methods

| Method          | Description                     |
| --------------- | ------------------------------- |
| `end(metadata)` | End span with optional metadata |

## Output Format

### TransactionRecord

```json
{
  "id": "uuid",
  "trace_id": "uuid",
  "name": "GET /users",
  "transaction_type": "request",
  "start_time": "2024-01-01T00:00:00Z",
  "end_time": "2024-01-01T00:00:00.050Z",
  "duration_ms": 50.0,
  "result": "HTTP 200",
  "correlation_id": "optional-correlation-id",
  "metadata": {},
  "service": {
    "service_name": "my-service",
    "service_version": "1.0.0",
    "environment": "production"
  }
}
```

### SpanRecord

```json
{
  "id": "uuid",
  "transaction_id": "uuid",
  "trace_id": "uuid",
  "parent_id": "optional-parent-uuid",
  "name": "database.query",
  "span_type": "db",
  "subtype": "mongodb",
  "start_time": "2024-01-01T00:00:00Z",
  "end_time": "2024-01-01T00:00:00.025Z",
  "duration_ms": 25.0,
  "metadata": {}
}
```

## Middleware Integration

```rust
use rusta_apm::apm_middleware;
use rusta::{App, MiddlewareChain};

let middleware = MiddlewareChain::new()
    .chain(apm_middleware);

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

- Creates a transaction for each request
- Extracts/propagates correlation IDs
- Records response status as transaction result
- Handles panics gracefully
