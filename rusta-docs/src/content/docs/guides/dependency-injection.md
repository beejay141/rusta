---
title: Dependency Injection
description: Manage service dependencies with the DI container.
sidebar:
  order: 2
---

# Dependency Injection

Rusta provides a DI container for managing service dependencies.

:::tip[Why DI?]
Dependency injection decouples your business logic from concrete implementations. Swap databases, mock services in tests, and configure different environments without changing controller code.
:::

## Basic Injection

Mark structs as injectable and declare dependencies:

```rust
use rusta::prelude::*;
use std::sync::Arc;

#[injectable]
pub struct UserService {
    #[inject]
    repo: Arc<dyn UserRepository>,
}

#[injectable]
pub struct UserController {
    #[inject]
    svc: Arc<UserService>,
}
```

## Registration

Register services in the container:

```rust
let mut container = Container::new();

// Register repository
container.register(Arc::new(MongoUserRepository::new()));

// Register service (dependencies resolved automatically)
container.register(UserService::construct(&container));

// Register controller
container.register(UserController::construct(&container));
```

:::tip[Registration Order]
Register dependencies before dependents. The container resolves at registration time, so `UserRepository` must be registered before `UserService`.
:::

## Optional Injection

Make dependencies optional - field becomes `None` if not registered:

```rust
#[injectable]
pub struct EmailService {
    #[inject(optional)]
    smtp_client: Option<Arc<dyn SmtpClient>>,
}

impl EmailService {
    pub fn send(&self, email: Email) -> Result<(), Error> {
        match &self.smtp_client {
            Some(client) => client.send(email).await,
            None => Err(Error::NotConfigured),
        }
    }
}
```

## Named Bindings

Register multiple implementations of the same trait:

```rust
// Define two cache implementations
container.register_named(Arc::new(RedisCache::new()), "redis");
container.register_named(Arc::new(MemoryCache::new()), "memory");

// Inject by name
#[injectable]
pub struct CacheService {
    #[inject(name = "redis")]
    primary: Arc<dyn Cache>,
    #[inject(name = "memory")]
    fallback: Arc<dyn Cache>,
}
```

## Resolution in Handlers

Use the `Inject` extractor to resolve dependencies in handlers:

```rust
#[get("/users")]
pub async fn list_users(
    Inject(svc): Inject<Arc<UserService>>,
) -> Response {
    Http::json(svc.list().await)
}
```

## Binding Verification

Verify all required bindings at startup:

```rust
let errors = container.verify();
if !errors.is_empty() {
    panic!("Missing bindings: {:?}", errors);
}
```

This catches missing dependencies before the server starts.

## Trait Objects

Use trait objects for abstraction:

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: &str) -> Option<User>;
    async fn save(&self, user: User) -> Result<User, Error>;
}

#[injectable]
pub struct UserService {
    #[inject]
    repo: Arc<dyn UserRepository>,
}
```

## Container Reference

For middleware that needs access to the container:

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
