---
title: CORS Configuration
description: Configure Cross-Origin Resource Sharing for your Ravix API.
sidebar:
  order: 2
---

# CORS Configuration

Ravix provides a `CorsConfig` builder for configuring Cross-Origin Resource Sharing.

## Basic Configuration

```rust
use ravix::CorsConfig;

let cors = CorsConfig::builder()
    .allow_origins(vec!["https://app.example.com".to_string()])
    .allow_methods(vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()])
    .allow_headers(vec!["content-type".to_string(), "authorization".to_string()])
    .max_age(3600)
    .build();

App::new()
    .container(container)
    .cors(cors)
    .await;
```

## CorsConfigBuilder Methods

| Method                        | Description                                |
| ----------------------------- | ------------------------------------------ |
| `allow_origins(Vec<String>)`  | Allowed origins (use `["*"]` for all)      |
| `allow_methods(Vec<String>)`  | Allowed HTTP methods                       |
| `allow_headers(Vec<String>)`  | Allowed request headers                    |
| `expose_headers(Vec<String>)` | Headers exposed to the browser             |
| `allow_credentials(bool)`     | Allow cookies/credentials (default: false) |
| `max_age(seconds)`            | Cache preflight response (default: 86400)  |
| `build()`                     | Build the configuration                    |

## Common Configurations

### Development (Allow All)

```rust
let cors = CorsConfig::builder()
    .allow_origins(vec!["*".to_string()])
    .allow_methods(vec!["*".to_string()])
    .allow_headers(vec!["*".to_string()])
    .allow_credentials(true)
    .build();
```

### Production (Restrictive)

```rust
let cors = CorsConfig::builder()
    .allow_origins(vec![
        "https://app.example.com".to_string(),
        "https://admin.example.com".to_string(),
    ])
    .allow_methods(vec!["GET".to_string(), "POST".to_string()])
    .allow_headers(vec!["content-type".to_string(), "authorization".to_string()])
    .expose_headers(vec!["x-request-id".to_string()])
    .max_age(3600)
    .build();
```

### Multiple Environments

```rust
fn cors_config(env: &str) -> CorsConfig {
    match env {
        "development" => CorsConfig::builder()
            .allow_origins(vec!["http://localhost:3000".to_string()])
            .allow_methods(vec!["*".to_string()])
            .allow_headers(vec!["*".to_string()])
            .allow_credentials(true)
            .build(),
        "production" => CorsConfig::builder()
            .allow_origins(vec!["https://app.example.com".to_string()])
            .allow_methods(vec!["GET".to_string(), "POST".to_string()])
            .allow_headers(vec!["content-type".to_string(), "authorization".to_string()])
            .build(),
        _ => CorsConfig::builder().build(),
    }
}
```

## How It Works

The CORS middleware:

1. Handles `OPTIONS` preflight requests automatically
2. Adds `Access-Control-Allow-*` headers to responses
3. Validates origin against allowed list
4. Rejects requests from disallowed origins with 403

:::caution[Credentials + Wildcard]
When `allow_credentials(true)`, you cannot use `allow_origins(vec!["*"])`. Specify exact origins.
:::

## Testing CORS

```bash
# Preflight request
curl -X OPTIONS http://localhost:3000/users \
  -H "Origin: https://app.example.com" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: content-type" \
  -v

# Actual request
curl -X POST http://localhost:3000/users \
  -H "Origin: https://app.example.com" \
  -H "Content-Type: application/json" \
  -d '{"name": "Test"}' \
  -v
```
