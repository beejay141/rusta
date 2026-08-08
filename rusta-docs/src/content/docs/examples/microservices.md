---
title: Microservices Patterns
description: Multi-service patterns and shared library strategies.
sidebar:
  order: 2
---

# Microservices Patterns

Patterns for building multi-service Rusta applications.

## Shared Kernel Pattern

Extract common code to a shared crate:

```
workspace/
├── Cargo.toml
├── shared/
│   ├── Cargo.toml
│   └── src/
│       ├── models/
│       ├── errors/
│       └── di/
├── user-service/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── controllers/
└── post-service/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        └── controllers/
```

### Shared Crate

```rust
// shared/src/lib.rs
pub mod models;
pub mod errors;
pub mod di;

// shared/src/models/user.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

// shared/src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SharedError {
    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Invalid request: {0}")]
    BadRequest(String),
}

// shared/src/di.rs
use rusta::Container;

pub fn build_shared_container() -> Container {
    let mut container = Container::new();
    // Shared services
    container
}
```

## Service-to-Service Communication

### HTTP Clients

```rust
// shared/src/http/client.rs
use reqwest::Client;
use serde_json::Value;

pub struct ServiceClient {
    client: Client,
    base_url: String,
}

impl ServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User, reqwest::Error> {
        self.client
            .get(format!("{}/users/{}", self.base_url, id))
            .send()
            .await?
            .json()
            .await
    }
}
```

### Message Queues

```rust
// shared/src/events.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    UserCreated { id: Uuid, email: String },
    PostPublished { id: Uuid, author_id: Uuid },
}

// In service
use redis::AsyncCommands;

async fn publish_event(event: Event) -> Result<(), redis::RedisError> {
    let mut conn = redis::connect("redis://localhost").await?;
    let payload = serde_json::to_string(&event)?;
    conn.publish("events", payload).await?;
    Ok(())
}
```

## Configuration Per Service

```rust
// user-service/src/config.rs
use config::{Config, Environment};

#[derive(Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub database_url: String,
    pub event_bus_url: String,
}

impl ServiceConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(Environment::with_prefix("USER_SERVICE"))
            .build()?
            .try_deserialize()
    }
}
```

## Database Per Service

Each service owns its data:

```rust
// user-service/src/db.rs
use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
}

// post-service/src/db.rs - separate database
```

## Health Check Aggregation

```rust
// shared/src/health.rs
use reqwest::Client;

pub async fn check_all_services(services: &[&str]) -> Vec<(String, bool)> {
    let client = Client::new();
    let mut results = Vec::new();

    for service in services {
        let url = format!("{}/health", service);
        let healthy = client.get(&url).send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        results.push((url, healthy));
    }

    results
}
```

## Docker Compose for Multiple Services

```yaml
# docker-compose.yml
version: "3.8"
services:
  user-service:
    build: ./user-service
    ports: ["3001:3000"]
    environment:
      - DATABASE_URL=postgres://user:pass@user-db:5432/users
      - EVENT_BUS_URL=redis://redis:6379

  post-service:
    build: ./post-service
    ports: ["3002:3000"]
    environment:
      - DATABASE_URL=postgres://post:pass@post-db:5432/posts
      - EVENT_BUS_URL=redis://redis:6379

  user-db:
    image: postgres:16
    environment:
      - POSTGRES_DB=users

  post-db:
    image: postgres:16
    environment:
      - POSTGRES_DB=posts

  redis:
    image: redis:7-alpine
```

## API Gateway Pattern

```rust
// gateway/src/main.rs
use rusta::{App, MiddlewareChain};
use tower_http::cors::CorsLayer;

let app = App::new()
    .middleware(MiddlewareChain::new()
        .add_layer(CorsLayer::permissive())
    );

// Route to services based on path
// /users/* -> user-service
// /posts/* -> post-service
```

## Best Practices

:::tip[One Service, One Database]
Each microservice should own its data exclusively. Never share databases between services.
:::

:::caution[Event Versioning]
Version your events when services evolve independently. Use semantic versioning for event schemas.
:::

:::note[Shared Library Updates]
When updating the shared crate, all services must be updated together. Consider API versioning for backward compatibility.
:::
