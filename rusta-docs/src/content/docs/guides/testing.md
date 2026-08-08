---
title: Testing
description: Unit test handlers, integration test with testcontainers, and mock services.
sidebar:
  order: 5
---

# Testing

Rusta applications are designed for testability — DI container makes mocking straightforward.

## Unit Testing Handlers

Test controllers in isolation by injecting mock services:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusta::{App, Container, Http, Response};
    use std::sync::Arc;
    use mockall::predicate::*;

    // Mock your service trait
    mock! {
        UserService {}
        impl UserService {
            async fn list(&self) -> Vec<User>;
            async fn find(&self, id: &str) -> Option<User>;
            async fn create(&self, dto: CreateUserDto) -> User;
        }
    }

    #[tokio::test]
    async fn test_list_users() {
        let mut mock_svc = MockUserService::new();
        mock_svc.expect_list()
            .returning(|| vec![User { id: "1".into(), name: "Alice".into() }]);

        let mut container = Container::new();
        container.register(Arc::new(mock_svc));

        let controller = UserController::construct(&container);
        let response = controller.list().await;

        assert_eq!(response.status(), 200);
        // Parse JSON body and assert
    }
}
```

## Integration Testing with Testcontainers

Spin up real dependencies (PostgreSQL, MongoDB, Redis) for integration tests:

```rust
// tests/integration/user_api.rs
use testcontainers::{runners::AsyncRunner, GenericImage};
use rusta::{App, Container};
use reqwest::Client;

#[tokio::test]
async fn test_user_crud() {
    // Start PostgreSQL container
    let postgres = GenericImage::new("postgres", "16")
        .with_env_var("POSTGRES_DB", "testdb")
        .with_env_var("POSTGRES_USER", "test")
        .with_env_var("POSTGRES_PASSWORD", "test")
        .start()
        .await
        .unwrap();

    let port = postgres.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://test:test@localhost:{}/testdb", port);

    // Build app with real database
    let container = build_test_container(&db_url).await;
    let app = App::new().container(container);
    let addr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app.into_router()).await.unwrap();
    });

    let client = Client::new();
    let base = format!("http://{}", server_addr);

    // Test CREATE
    let create_resp = client
        .post(&format!("{}/users", base))
        .json(&json!({ "name": "Bob", "email": "bob@example.com" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_resp.status(), 201);
    let user: User = create_resp.json().await.unwrap();

    // Test GET
    let get_resp = client
        .get(&format!("{}/users/{}", base, user.id))
        .send()
        .await
        .unwrap();
    assert_eq!(get_resp.status(), 200);
    let fetched: User = get_resp.json().await.unwrap();
    assert_eq!(fetched.name, "Bob");
}
```

## Mocking the DI Container

For fast unit tests, swap implementations in the container:

```rust
fn build_test_container() -> Container {
    let mut container = Container::new();

    // Use in-memory implementations
    container.register(Arc::new(InMemoryUserRepository::new()));
    container.register(UserService::construct(&container));
    container.register(UserController::construct(&container));

    container
}
```

## Testing Middleware

Test middleware in isolation:

```rust
#[tokio::test]
async fn test_auth_middleware_rejects_missing_token() {
    let middleware = auth_middleware;
    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = middleware(request, next).await;
    assert_eq!(response.status(), 401);
}
```

## Test Organization

```
tests/
├── unit/
│   ├── controllers/
│   │   └── user_controller_test.rs
│   └── services/
│       └── user_service_test.rs
├── integration/
│   ├── user_api_test.rs
│   └── auth_test.rs
└── fixtures/
    └── test_data.json
```

## Running Tests

```bash
# Unit tests only (fast)
cargo test --lib

# Integration tests (requires Docker)
cargo test --test integration

# All tests with output
cargo test -- --nocapture
```

## Best Practices

:::tip[Test the Contract, Not Implementation]
Test handler behavior (status codes, response shape) not internal service calls.
:::

:::caution[Parallel Test Isolation]
Each integration test should get its own database. Use testcontainers with random ports or separate schemas.
:::

:::note[DI Makes Testing Easy]
The container pattern means you never need to mock the framework — just swap the service implementation.
Add `mockall` and `testcontainers` to dev-dependencies in Cargo.toml:

```toml
[dev-dependencies]
mockall = "0.12"
testcontainers = { version = "0.19", features = ["tokio"] }
reqwest = { version = "0.12", features = ["json"] }
```

:::
