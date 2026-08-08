---
title: API Reference
description: Complete reference for all public Rusta types and methods.
sidebar:
  order: 5
---

# API Reference

Complete reference for all public Rusta APIs.

## Core Types

### `App`

The main application builder.

```rust
use rusta::App;

let app = App::new()
    .container(container)
    .middleware(middleware)
    .cors(cors)
        .run("0.0.0.0:3000", |res| match res {
            Ok(addr) => println!("Started on {}", addr),
            Err(msg) => eprintln!("Startup error: {}", msg),
        })
    .await;
```

| Method                        | Description               |
| ----------------------------- | ------------------------- |
| `new()`                       | Create a new App instance |
| `container(Container)`        | Set the DI container      |
| `middleware(MiddlewareChain)` | Add middleware            |
| `cors(CorsConfig)`            | Configure CORS            |
| `run(&str)`                   | Start the server          |

### `Container`

Dependency injection container.

```rust
use rusta::Container;

let mut container = Container::new();
container.register(MyService::construct(&container));
container.verify()?;
```

| Method                    | Description                |
| ------------------------- | -------------------------- |
| `new()`                   | Create a new container     |
| `register(T)`             | Register a service         |
| `register_named(T, &str)` | Register with a name       |
| `verify()`                | Check for missing bindings |

### `Http`

Response helper methods.

```rust
use rusta::Http;

Http::json(data)           // 200 OK with JSON
Http::created(data)         // 201 Created
Http::no_content()          // 204 No Content
Http::not_found(msg)        // 404 Not Found
Http::unauthorized(msg)     // 401 Unauthorized
Http::bad_request(msg)      // 400 Bad Request
```

## Proc-Macros

### `#[controller(path)]`

Register a controller with routes.

```rust
#[controller("/users")]
impl UserController {
    #[get("/")]
    async fn list(&self) -> Response { ... }

    #[post("/")]
    async fn create(&self, Json(body): Json<Dto>) -> Response { ... }
}
```

### `#[injectable]`

Mark a struct for DI registration.

```rust
#[injectable]
pub struct UserService {
    #[inject]
    repo: Arc<dyn UserRepository>,
}
```

### `#[inject]`

Inject a dependency.

```rust
#[injectable]
pub struct UserController {
    #[inject]
    svc: Arc<UserService>,
}
```

### `#[middleware(fn)]`

Attach middleware to a handler.

```rust
#[post("/")]
#[middleware(auth_guard)]
async fn create(&self, ...) -> Response { ... }
```

## HTTP Method Attributes

| Attribute          | Method  |
| ------------------ | ------- |
| `#[get(path)]`     | GET     |
| `#[post(path)]`    | POST    |
| `#[put(path)]`     | PUT     |
| `#[patch(path)]`   | PATCH   |
| `#[delete(path)]`  | DELETE  |
| `#[head(path)]`    | HEAD    |
| `#[options(path)]` | OPTIONS |

## Extractors

All standard axum extractors work:

```rust
use axum::{Json, Path, Query, Extension, State};

async fn handler(
    Json(body): Json<Dto>,
    Path(id): Path<String>,
    Query(params): Query<Pagination>,
    Extension(claims): Extension<Claims>,
    State(container): State<ContainerRef>,
) -> Response { ... }
```

## Middleware Types

### `MiddlewareChain`

Build middleware pipelines.

```rust
use rusta::MiddlewareChain;

let middleware = MiddlewareChain::new()
    .chain(my_middleware)
    .add_layer(CompressionLayer::new());
```

| Method             | Description             |
| ------------------ | ----------------------- |
| `chain(fn)`        | Add middleware function |
| `add_layer(Layer)` | Add Tower layer         |

### `Request`, `Next`, `Response`

Standard axum types for middleware.

```rust
pub async fn my_middleware(
    request: Request,
    next: Next,
) -> Response {
    // ...
    next.run(request).await
}
```

## Error Types

### `ErrorResponse`

```rust
pub enum ErrorResponse {
    Message(String),  // { "error": "message" }
    Object(Value),    // Pass-through JSON
}
```

### `ErrorObject`

```rust
use rusta::ErrorObject;

Http::bad_request(ErrorObject(json!({
    "code": "VALIDATION_ERROR",
    "fields": ["email"]
})))
```

## Prelude

```rust
use rusta::prelude::*;

// Includes:
// - App, Container, Http
// - Response, Request, Next
// - All HTTP method macros
// - All extractor types
```
