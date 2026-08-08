---
title: Project Structure
description: Recommended folder layout for Ravix applications.
sidebar:
  order: 3
---

# Project Structure

Ravix doesn't enforce a specific structure, but this layout scales well for production applications.

## Recommended Layout

```
my-api/
├── Cargo.toml
├── src/
│   ├── main.rs                 # App bootstrap
│   ├── config.rs               # Configuration loading
│   ├── error.rs                # Custom error types
│   ├── controllers/            # HTTP layer
│   │   ├── mod.rs
│   │   ├── user_controller.rs
│   │   └── post_controller.rs
│   ├── services/               # Business logic
│   │   ├── mod.rs
│   │   ├── user_service.rs
│   │   └── post_service.rs
│   ├── repositories/           # Data access
│   │   ├── mod.rs
│   │   ├── user_repository.rs
│   │   └── post_repository.rs
│   ├── models/                 # Domain models & DTOs
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   └── post.rs
│   ├── middleware/             # Custom middleware
│   │   ├── mod.rs
│   │   └── auth.rs
│   └── di/                     # DI container setup
│       └── container.rs
├── tests/
│   ├── integration/
│   └── fixtures/
└── docker/
    ├── Dockerfile
    └── docker-compose.yml
```

## Layer Responsibilities

| Layer            | Responsibility                                    | Example                 |
| ---------------- | ------------------------------------------------- | ----------------------- |
| **Controllers**  | HTTP concerns: routing, extraction, serialization | `UserController`        |
| **Services**     | Business logic, orchestration, transactions       | `UserService`           |
| **Repositories** | Data access, queries, persistence                 | `UserRepository`        |
| **Models**       | Domain entities, DTOs, request/response types     | `User`, `CreateUserDto` |

## Module Organization

```rust
// src/controllers/mod.rs
pub mod user_controller;
pub mod post_controller;

// src/services/mod.rs
pub mod user_service;
pub mod post_service;

// src/repositories/mod.rs
pub mod user_repository;
pub mod post_repository;

// src/models/mod.rs
pub mod user;
pub mod post;
```

## DI Container Setup

```rust
// src/di/container.rs
use ravix::Container;
use crate::{
    services::{UserService, PostService},
    repositories::{UserRepository, PostRepository},
};

pub fn build_container() -> Container {
    let mut container = Container::new();

    // Repositories (no dependencies)
    container.register(Arc::new(MongoUserRepository::new()));
    container.register(Arc::new(MongoPostRepository::new()));

    // Services (depend on repositories)
    container.register(UserService::construct(&container));
    container.register(PostService::construct(&container));

    // Controllers (depend on services)
    container.register(UserController::construct(&container));
    container.register(PostController::construct(&container));

    container.verify().expect("Missing DI bindings");

    container
}
```

```rust
// src/main.rs
mod controllers;
mod services;
mod repositories;
mod models;
mod di;

use ravix::App;
use di::build_container;

#[tokio::main]
async fn main() {
    let container = build_container();
    App::new().container(container).run("0.0.0.0:3000", |res| match res {
        Ok(addr) => println!("Listening on {}", addr),
        Err(msg) => eprintln!("Startup error: {}", msg),
    }).await;
}
```

## Scaling Tips

1. **Feature folders** — For large apps, group by feature instead of layer:

   ```
   src/
   ├── users/
   │   ├── controller.rs
   │   ├── service.rs
   │   ├── repository.rs
   │   └── model.rs
   └── posts/
       ├── controller.rs
       └── ...
   ```

2. **Shared kernel** — Extract common types to a `shared` crate for multi-service projects.

3. **Test isolation** — Keep integration tests in `tests/` with their own DI setup.
