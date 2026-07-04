# Ravix

A modern Rust API framework built on [axum](https://github.com/tokio-rs/axum) with clean-architecture patterns, proc-macro route registration, and InversifyJS-style dependency injection.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-blue.svg)](https://www.rust-lang.org)

## Features

- **Clean Architecture**: Controller → Service → Repository layers with clear separation of concerns
- **Proc-macro Routing**: Declarative route registration with `#[controller]` and HTTP method attributes
- **Dependency Injection**: InversifyJS-inspired DI container with `#[injectable]` and `#[inject]` attributes
- **Per-handler Middleware**: Attach middleware to individual handlers with `#[middleware]` attribute
- **Optional Global Middleware**: Register CORS and other security middleware at the `App` level — each feature is opt-in
- **Axum Abstraction**: Clean type aliases for common axum types (no direct `axum::` imports needed)
- **Async-First**: Built for async/await with Tokio runtime

## Quick Start

```toml
# Cargo.toml
[dependencies]
ravix = { path = "ravix" }
ravix-macros = { path = "ravix-macros" }
```

```rust
use ravix::prelude::*;
use std::sync::Arc;

// Define your controller
#[controller("/users")]
impl UserController {
    #[get("/")]
    pub async fn list_users(&self) -> Response {
        Http::json(self.svc.find_all().await)
    }

    #[get("/:id")]
    pub async fn get_user(&self, Path(id): Path<Uuid>) -> Response {
        match self.svc.find_by_id(id).await {
            Some(user) => Http::json(user),
            None => Http::not_found("User not found"),
        }
    }

    #[post("/")]
    pub async fn create_user(&self, Json(body): Json<CreateUserDto>) -> Response {
        Http::created(self.svc.create(body).await)
    }
}

// Bootstrap
#[tokio::main]
async fn main() {
    let mut container = Container::new();
    // Register dependencies...
    App::new().container(container).run("0.0.0.0:3000").await;
}
```

## Architecture

```
ravix/
├── ravix/           # Framework core
│   ├── app.rs       # Application builder
│   ├── container.rs # DI container
│   ├── handler.rs   # Route descriptor
│   ├── middleware/  # Middleware utilities
│   ├── request.rs   # Request wrapper
│   └── response.rs  # Response helpers
├── ravix-macros/    # Proc-macros
│   ├── controller.rs # #[controller] attribute
│   ├── routes.rs     # #[get], #[post], etc.
│   ├── middleware_attr.rs # #[middleware] attribute
│   └── injectable.rs # #[injectable] attribute
└── ravix-example/   # Reference application
```

## API Reference

### Route Macros

| Macro                    | Description                                        |
| ------------------------ | -------------------------------------------------- |
| `#[controller("/base")]` | Marks an impl block as a controller with base path |
| `#[get("/path")]`        | HTTP GET route                                     |
| `#[post("/path")]`       | HTTP POST route                                    |
| `#[put("/path")]`        | HTTP PUT route                                     |
| `#[delete("/path")]`     | HTTP DELETE route                                  |
| `#[patch("/path")]`      | HTTP PATCH route                                   |
| `#[middleware(Guard)]`   | Attach middleware to a handler                     |

### Response Helpers

```rust
Http::json(data)           // 200 OK with JSON body
Http::created(data)        // 201 Created with JSON body
Http::ok()                 // 200 OK with empty body
Http::not_found("msg")     // 404 Not Found with error JSON
Http::unauthorized("msg")  // 401 Unauthorized with error JSON
Http::bad_request("msg")   // 400 Bad Request with error JSON
Http::internal_error("msg")  // 500 Internal Server Error with error JSON
Http::status(code)         // Arbitrary status code
```

### Dependency Injection

```rust
// Mark a struct as injectable
#[injectable]
pub struct UserService {
    #[inject]
    pub repo: Arc<dyn UserRepository>,
}

// Register in container
let mut container = Container::new();
container.register(Arc::new(InMemoryUserRepository::default()));
container.register(UserService::construct(&container));

// Resolve in handlers
pub async fn get_user(Inject(svc): Inject<Arc<UserService>>) -> Response {
    // ...
}
```

### Prelude Exports

The `ravix::prelude::*` glob import provides:

```rust
// HTTP method macros
controller, get, post, put, delete, patch, middleware, injectable

// DI types
Container, ContainerRef, Inject, Injectable

// Framework types
Http, Json, Path, Query, State, Body, Request, Response, IntoResponse, StatusCode
```

### CORS

CORS is opt-in and registered globally on the `App` builder. It is applied to every route, including `OPTIONS` preflight requests, before any per-handler middleware runs.

```rust
use ravix::{App, Container, CorsConfig};

let cors = CorsConfig::builder()
    .allow_origins(vec!["https://app.example.com".to_string()])
    .allow_methods(vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()])
    .allow_headers(vec!["content-type".to_string(), "authorization".to_string()])
    .max_age(3600)   // seconds to cache preflight response
    .build();

App::new()
    .container(container)
    .cors(cors)      // omit to disable CORS entirely
    .run("0.0.0.0:3000")
    .await;
```

**`CorsConfig` builder options:**

| Method                       | Description                                               |
| ---------------------------- | --------------------------------------------------------- |
| `.allow_origins(vec![...])`  | Explicit allowed origins. Omit to allow any origin (`*`). |
| `.allow_methods(vec![...])`  | Allowed HTTP methods. Omit to allow any.                  |
| `.allow_headers(vec![...])`  | Allowed request headers. Omit to allow any.               |
| `.expose_headers(vec![...])` | Response headers exposed to the browser.                  |
| `.max_age(seconds)`          | How long browsers may cache the preflight response.       |
| `.allow_credentials()`       | Enable `Access-Control-Allow-Credentials: true`.          |

> **Security note**: When `allow_credentials()` is enabled you **must** specify explicit `allow_origins()` and `allow_headers()`. Combining credentials with a wildcard (`*`) is invalid per the CORS specification and will panic at startup. If you use cookie-based auth alongside credentials, also add CSRF protections.

## Development

```bash
# Check all crates
cargo check --workspace

# Run tests
cargo test --workspace

# Run the example
cargo run -p ravix-example
```

## License

MIT
