---
title: Quick Start
description: Build your first Ravix API in 5 minutes.
sidebar:
  order: 2
---

# Quick Start

Build your first Ravix API in 5 minutes.

## 1. Create a New Project

```bash
cargo new my-api --bin
cd my-api
```

## 2. Add Dependencies

```bash
# Add to Cargo.toml
cargo add ravix tokio --features full
```

## 3. Create Your First Controller

Create `src/controllers/user_controller.rs`:

```rust
use ravix::prelude::*;
use std::sync::Arc;

#[injectable]
pub struct UserService {
    // Add your dependencies here
}

#[controller("/users")]
impl UserController {
    #[get("/")]
    pub async fn list(&self) -> Response {
        Http::json(json!([{ "id": 1, "name": "Alice" }]))
    }

    #[get("/:id")]
    pub async fn get(&self, Path(id): Path<String>) -> Response {
        Http::json(json!({ "id": id, "name": "User" }))
    }

    #[post("/")]
    pub async fn create(&self, Json(body): Json<serde_json::Value>) -> Response {
        Http::created(body)
    }
}
```

## 4. Bootstrap the Application

Update `src/main.rs`:

```rust
mod controllers;

use ravix::{App, Container};
use controllers::UserController;

#[tokio::main]
async fn main() {
    let mut container = Container::new();
    container.register(UserController::construct(&container));

    App::new()
        .container(container)
        .run("0.0.0.0:3000", |res| match res {
            Ok(addr) => println!("Listening on {}", addr),
            Err(msg) => eprintln!("Startup error: {}", msg),
        })
        .await;
}
```

Create `src/controllers/mod.rs`:

```rust
pub mod user_controller;
```

## 5. Run and Test

```bash
cargo run

# Test endpoints
curl http://localhost:3000/users
curl http://localhost:3000/users/123
curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d '{"name":"Bob"}'
```

## Next Steps

- [Controllers Guide](/guides/controllers) — Learn route patterns
- [Dependency Injection](/guides/dependency-injection) — Add services
- [Middleware](/guides/middleware) — Add authentication

---

<nav class="pagination">
  <a class="pagination-link prev" href="/getting-started/installation">← Installation</a>
  <a class="pagination-link next" href="/getting-started/project-structure">Project Structure →</a>
</nav>
