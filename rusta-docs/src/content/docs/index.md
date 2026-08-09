---
title: Rusta Documentation
template: splash
hero:
  title: Build Modern Rust APIs
  tagline: A clean-architecture framework with dependency injection, declarative routing, and built-in observability.
  actions:
    - text: Get Started
      link: /getting-started/
      icon: right-arrow
      variant: primary
    - text: View on GitHub
      link: https://github.com/beejay141/rusta
      icon: external
---

<div class="feature-grid">
  <div class="feature-card">
    <div class="feature-icon">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2L2 7l10 5 10-5-10-5z"/>
        <path d="M2 17l10 5 10-5"/>
        <path d="M2 12l10 5 10-5"/>
      </svg>
    </div>
    <h3>Clean Architecture</h3>
    <p>Controller → Service → Repository layers with clear separation of concerns</p>
  </div>
  
  <div class="feature-card">
    <div class="feature-icon">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6-1.6a1 1 0 0 0-1.4-1.4l1.6 1.6zM4 10h8"/>
        <path d="M4 14h8"/>
        <path d="M18 4v16"/>
      </svg>
    </div>
    <h3>Declarative Routing</h3>
    <p>`#[controller]`, `#[get]`, `#[post]` — routes register themselves</p>
  </div>
  
  <div class="feature-card">
    <div class="feature-icon">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <path d="M12 16v-4"/>
        <path d="M12 8h.01"/>
      </svg>
    </div>
    <h3>Built-in Observability</h3>
    <p>APM tracing and structured logging — opt-in, zero-config</p>
  </div>
</div>

## Quick Comparison

```rust
// Raw Axum — boilerplate you write every project
let app = Router::new()
    .route("/users", get(list_users).post(create_user))
    .layer(Extension(Arc::new(UserService::new())))
    .layer(CorsLayer::permissive());

// Rusta — declare intent, framework handles wiring
#[controller("/users")]
impl UserController {
    #[get("/")]
    async fn list(&self) -> Response { Http::json(self.svc.list().await) }

    #[post("/")]
    async fn create(&self, Json(body): Json<CreateUserDto>) -> Response {
        Http::created(self.svc.create(body).await)
    }
}
```

## Quick Start

```bash
# Install the CLI
cargo install cargo-rusta

# Scaffold a new project
cargo rusta new my-api
cd my-api

# Run it
cargo run
```

Or add Rusta to an existing project — see the [Installation guide](/getting-started/installation).

## Quick Example

```rust
use rusta::prelude::*;
use std::sync::Arc;

#[controller("/users")]
impl UserController {
    #[get("/")]
    pub async fn list(&self) -> Response {
        Http::json(self.svc.find_all().await)
    }

    #[post("/")]
    pub async fn create(&self, Json(body): Json<CreateUserDto>) -> Response {
        Http::created(self.svc.create(body).await)
    }
}

#[tokio::main]
async fn main() {
    let mut container = Container::new();
    container.register(UserService::construct(&container));
    App::new().container(container).run("0.0.0.0:3000", |res| match res {
      Ok(addr) => println!("Started on {}", addr),
      Err(msg) => eprintln!("Startup error: {}", msg),
    }).await;
}
```

## Ecosystem

Rusta is a family of crates — use what you need, skip what you don't.

| Crate          | Purpose                                          | Status    |
| -------------- | ------------------------------------------------ | --------- |
| `rusta`        | Core framework: routing, DI, middleware          | ✅ Stable |
| `rusta-apm`    | Distributed tracing, spans, transactions         | ✅ Stable |
| `rusta-logger` | Structured logging with context propagation      | ✅ Stable |
| `rusta-di`     | Standalone DI container                          | ✅ Stable |
| `rusta-macros` | Proc-macros for `#[controller]`, `#[injectable]` | ✅ Stable |

## Next Steps

- **New to Rusta?** → [Installation](/getting-started/installation) → [Quick Start](/getting-started/quick-start)
- **Coming from Axum?** → [Controllers Guide](/guides/controllers) → [Dependency Injection](/guides/dependency-injection)
- **Building for production?** → [Error Handling](/guides/error-handling) → [Testing](/guides/testing) → [Deployment](/guides/deployment)
