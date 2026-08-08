---
title: Blog API Walkthrough
description: Complete walkthrough of the Ravix Blog API example application.
sidebar:
  order: 1
---

# Blog API Walkthrough

This guide walks through the [rusta-example](https://github.com/beejay141/rusta/tree/main/rusta-example) — a complete blog API demonstrating real-world Rusta patterns.

## Project Structure

```
ravix-example/
├── Cargo.toml
├── src/
│   ├── main.rs              # App bootstrap
│   ├── config.rs            # Configuration
│   ├── db.rs                # Database connection
│   ├── errors.rs            # Custom error types
│   ├── middleware.rs        # Custom middleware
│   ├── controllers/         # HTTP layer
│   │   ├── mod.rs
│   │   ├── post_controller.rs
│   │   └── user_controller.rs
│   ├── models/              # Domain models & DTOs
│   │   ├── mod.rs
│   │   ├── post.rs
│   │   └── user.rs
│   ├── repositories/        # Data access
│   │   ├── mod.rs
│   │   ├── post_repository.rs
│   │   └── user_repository.rs
│   └── services/            # Business logic
│       ├── mod.rs
│       ├── post_service.rs
│       └── user_service.rs
├── tests/
│   └── integration/
└── benches/
    └── blog_benchmarks.rs
```

## Key Components

### 1. Configuration (`config.rs`)

```rust
use config::{Config, File, Environment};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }
}
```

### 2. Models (`models/post.rs`)

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreatePostDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(min = 1))]
    pub content: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdatePostDto {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,

    #[validate(length(min = 1))]
    pub content: Option<String>,

    pub published: Option<bool>,
}
```

### 3. Repository (`repositories/post_repository.rs`)

```rust
use sqlx::PgPool;
use crate::models::post::{Post, CreatePostDto, UpdatePostDto};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, author_id: Uuid, dto: CreatePostDto) -> Result<Post, Error>;
    async fn find(&self, id: Uuid) -> Result<Option<Post>, Error>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Post>, Error>;
    async fn update(&self, id: Uuid, dto: UpdatePostDto) -> Result<Post, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

pub struct PostgresPostRepository {
    pool: PgPool,
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, author_id: Uuid, dto: CreatePostDto) -> Result<Post, Error> {
        sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, published, created_at, updated_at
            "#,
            dto.title, dto.content, author_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)
    }
    // ... other methods
}
```

### 4. Service (`services/post_service.rs`)

```rust
#[injectable]
pub struct PostService {
    #[inject]
    repo: Arc<dyn PostRepository>,
}

impl PostService {
    pub async fn create(&self, author_id: Uuid, dto: CreatePostDto) -> Result<Post, AppError> {
        dto.validate()?;
        self.repo.create(author_id, dto).await.map_err(AppError::from)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Post>, AppError> {
        self.repo.list(limit, offset).await.map_err(AppError::from)
    }

    pub async fn get(&self, id: Uuid) -> Result<Post, AppError> {
        self.repo.find(id).await?
            .ok_or(AppError::NotFound(format!("Post {}", id)))
    }
}
```

### 5. Controller (`controllers/post_controller.rs`)

```rust
#[injectable]
pub struct PostController {
    #[inject]
    svc: Arc<PostService>,
}

#[controller("/posts")]
impl PostController {
    #[get("/")]
    pub async fn list(
        &self,
        Query(params): Query<Pagination>,
    ) -> Result<Response, AppError> {
        let posts = self.svc.list(params.limit, params.offset).await?;
        Ok(Http::json(posts))
    }

    #[get("/:id")]
    pub async fn get(&self, Path(id): Path<Uuid>) -> Result<Response, AppError> {
        let post = self.svc.get(id).await?;
        Ok(Http::json(post))
    }

    #[post("/")]
    #[middleware(auth_guard)]
    pub async fn create(
        &self,
        Extension(claims): Extension<Claims>,
        Json(body): Json<CreatePostDto>,
    ) -> Result<Response, AppError> {
        let post = self.svc.create(claims.sub, body).await?;
        Ok(Http::created(post))
    }

    #[put("/:id")]
    #[middleware(auth_guard)]
    pub async fn update(
        &self,
        Extension(claims): Extension<Claims>,
        Path(id): Path<Uuid>,
        Json(body): Json<UpdatePostDto>,
    ) -> Result<Response, AppError> {
        let post = self.svc.update(claims.sub, id, body).await?;
        Ok(Http::json(post))
    }

    #[delete("/:id")]
    #[middleware(auth_guard)]
    pub async fn delete(
        &self,
        Extension(claims): Extension<Claims>,
        Path(id): Path<Uuid>,
    ) -> Result<Response, AppError> {
        self.svc.delete(claims.sub, id).await?;
        Ok(Http::no_content())
    }
}
```

### 6. Bootstrap (`main.rs`)

```rust
mod config;
mod db;
mod errors;
mod middleware;
mod controllers;
mod models;
mod repositories;
mod services;

use ravix::{App, Container, MiddlewareChain};
use ravix_apm::apm_middleware;
use ravix_logger::logger_middleware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = config::Settings::new()?;

    // Database
    let pool = db::create_pool(&settings.database).await?;

    // DI Container
    let mut container = Container::new();
    container.register(Arc::new(pool));
    container.register(repositories::UserRepository::construct(&container));
    container.register(repositories::PostRepository::construct(&container));
    container.register(services::UserService::construct(&container));
    container.register(services::PostService::construct(&container));
    container.register(controllers::UserController::construct(&container));
    container.register(controllers::PostController::construct(&container));
    container.verify()?;

    // Middleware
    let middleware = MiddlewareChain::new()
        .chain(middleware::request_id)
        .chain(apm_middleware)
        .chain(logger_middleware);

    // App
    App::new()
        .container(container)
        .middleware(middleware)
        .run(&format!("{}:{}", settings.server.host, settings.server.port))
        .await?;

    Ok(())
}
```

## Running the Example

```bash
# From ravix-example directory
cd ravix-example

# Start PostgreSQL
docker-compose up -d db

# Run migrations
sqlx migrate run

# Start server
cargo run

# Test endpoints
curl http://localhost:3000/health
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'
```

## Key Patterns Demonstrated

| Pattern                                | Location                                     |
| -------------------------------------- | -------------------------------------------- |
| Clean architecture layers              | `controllers/`, `services/`, `repositories/` |
| DI container with trait objects        | `main.rs` container registration             |
| Custom error types with `IntoResponse` | `errors.rs`                                  |
| Per-handler auth middleware            | `post_controller.rs`                         |
| Structured logging + APM               | `main.rs` middleware chain                   |
| Configuration with environments        | `config.rs`                                  |
| Input validation with `validator`      | `models/post.rs`                             |
| Integration tests with testcontainers  | `tests/integration/`                         |

## Next Steps

- [Controllers Guide](/guides/controllers) — Deep dive on routing
- [Dependency Injection](/guides/dependency-injection) — DI container patterns
- [Error Handling](/guides/error-handling) — Custom error types
- [Testing](/guides/testing) — Unit and integration testing
- [Deployment](/guides/deployment) — Docker and production setup
