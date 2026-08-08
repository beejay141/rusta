---
title: Controllers
description: Define HTTP routes with declarative #[controller] and HTTP method macros.
sidebar:
  order: 1
---

# Controllers

Controllers handle HTTP requests and responses. Rusta uses proc-macros for declarative route registration.

:::tip[Controller vs Handler]
Controllers are structs with an `impl` block. Each method becomes a route handler. The `#[controller]` macro registers all methods with HTTP attributes at compile time.
:::

## Basic Controller

```rust
use rusta::prelude::*;
use std::sync::Arc;

#[controller("/users")]
impl UserController {
    #[get("/")]
    pub async fn list(&self) -> Response {
        Http::json(self.svc.list().await)
    }

    #[get("/:id")]
    pub async fn get(&self, Path(id): Path<String>) -> Response {
        match self.svc.find(&id).await {
            Some(user) => Http::json(user),
            None => Http::not_found("User not found"),
        }
    }

    #[post("/")]
    pub async fn create(&self, Json(body): Json<CreateUserDto>) -> Response {
        Http::created(self.svc.create(body).await)
    }

    #[put("/:id")]
    pub async fn update(&self, Path(id): Path<String>, Json(body): Json<UpdateUserDto>) -> Response {
        Http::json(self.svc.update(&id, body).await)
    }

    #[delete("/:id")]
    pub async fn delete(&self, Path(id): Path<String>) -> Response {
        Http::no_content()
    }
}
```

## Route Parameters

Extract path, query, and body parameters using axum extractors:

```rust
#[get("/:id")]
pub async fn get_user(
    Path(id): Path<String>,           // Path parameter
    Query(params): Query<Pagination>,  // Query parameters
    Json(body): Json<UpdateDto>,       // Request body
) -> Response {
    // ...
}
```

:::caution[Route Order Matters]
Routes are registered in definition order. More specific routes (e.g., `/users/me`) must come before catch-all routes (e.g., `/users/:id`) to avoid shadowing.
:::

## Request Extensions

Access request extensions (e.g., authenticated user from middleware):

```rust
#[get("/profile")]
pub async fn profile(
    Extension(claims): Extension<Claims>,
) -> Response {
    Http::json(self.svc.get_profile(&claims.sub).await)
}
```

## Per-Handler Middleware

Attach middleware to individual handlers:

```rust
#[post("/")]
#[middleware(auth_guard)]
pub async fn create_post(
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreatePostDto>,
) -> Response {
    Http::created(self.svc.create(&claims.sub, body).await)
}
```

## Controller Dependencies

Inject services via the DI container:

```rust
#[injectable]
pub struct UserController {
    #[inject]
    svc: Arc<UserService>,
}

#[controller("/users")]
impl UserController {
    // ...
}
```

## Multiple Controllers

Register multiple controllers in your bootstrap:

```rust
mod controllers {
    pub mod user_controller;
    pub mod post_controller;
}

use controllers::{UserController, PostController};

#[tokio::main]
async fn main() {
    let mut container = Container::new();
    container.register(UserController::construct(&container));
    container.register(PostController::construct(&container));
    // ...
}
```
