---
title: Middleware
description: Add per-handler and global middleware for authentication, logging, and more.
sidebar:
  order: 3
---

# Middleware

Ravix supports both per-handler and global middleware.

:::caution[Middleware vs Layers]

- **Middleware** (`.chain()`) — async functions with `Request, Next` signature, run in order
- **Layers** (`.add_layer()`) — Tower `Layer` types, wrap the entire router, run in reverse order
  :::

## Per-Handler Middleware

Attach middleware to individual handlers using the `#[middleware]` attribute:

```rust
use ravix::prelude::*;

pub async fn auth_guard(
    State(container): State<ContainerRef>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request.headers().get("Authorization");
    // ... validate token ...
    request.extensions_mut().insert(claims);
    next.run(request).await
}

#[controller("/posts")]
impl PostController {
    #[get("/:id")]
    #[middleware(auth_guard)]
    pub async fn get_post(&self, Path(id): Path<String>) -> Response {
        Http::json(self.svc.find(&id).await)
    }
}
```

## Global Middleware Chain

Apply middleware to all routes:

```rust
use ravix::{App, MiddlewareChain, Request, Next, Response};

async fn request_id_middleware(request: Request, next: Next) -> Response {
    let id = uuid::Uuid::new_v4().to_string();
    request.extensions_mut().insert(id.clone());
    let mut response = next.run(request).await;
    response.headers_mut().insert("X-Request-ID", id.parse().unwrap());
    response
}

let middleware = MiddlewareChain::new()
    .chain(request_id_middleware)
    .chain(apm_middleware)
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

## Tower Layers

Add tower-http layers (compression, tracing, etc.):

```rust
use ravix::MiddlewareChain;
use tower_http::{
    compression::CompressionLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};

let middleware = MiddlewareChain::new()
    .add_layer(CompressionLayer::new())
    .add_layer(TraceLayer::new_for_http())
    .add_layer(TimeoutLayer::new(Duration::from_secs(30)));
```

## Middleware Ordering

Layers are applied in registration order:

```rust
// First added = innermost (closest to handler)
// Last added = outermost (first to see request)
let middleware = MiddlewareChain::new()
    .chain(auth_guard)      // Runs first on request, last on response
    .chain(logger_middleware)
    .add_layer(TraceLayer::new_for_http()); // Runs last on request, first on response
```

:::tip[Visualizing Order]
Think of middleware as an onion: the request travels **inward** through layers, the response travels **outward**. The first `.chain()` is the innermost layer.
:::

## Accessing Container in Middleware

Use `State<ContainerRef>` to resolve services:

```rust
async fn auth_middleware(
    State(container): State<ContainerRef>,
    request: Request,
    next: Next,
) -> Response {
    let auth_svc: Arc<AuthService> = container.resolve();
    // ...
}
```

## Built-in Middleware

### CORS

```rust
use ravix::CorsConfig;

let cors = CorsConfig::builder()
    .allow_origins(vec!["https://app.example.com".to_string()])
    .allow_methods(vec!["GET".to_string(), "POST".to_string()])
    .allow_headers(vec!["content-type".to_string()])
    .max_age(3600)
    .build();

App::new().cors(cors).await;
```

### APM Middleware

```rust
use ravix_apm::apm_middleware;

let middleware = MiddlewareChain::new()
    .chain(apm_middleware);
```

### Logger Middleware

```rust
use ravix_logger::logger_middleware;

let middleware = MiddlewareChain::new()
    .chain(logger_middleware);
```
