---
title: Testing
description: Unit test handlers, end-to-end integration tests with testcontainers, and mock services.
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

## End-to-End Integration Tests with Real Services

For full end-to-end coverage, the recommended pattern is to run the **actual service
binary** inside a Docker container alongside its dependencies. This catches issues
that unit tests and in-process integration tests miss — networking, serialization,
JSON contracts, JWT validation, etc.

The `rusta-example` project ships a production-ready setup for this.

### Project Structure

```
tests/integration/
├── mod.rs              # Module declarations
├── setup/
│   └── mod.rs          # startServiceContainer() helper
├── auth_tests.rs       # /auth/register, /auth/login
├── post_tests.rs       # /posts CRUD
├── comment_tests.rs    # /posts/:id/comments CRUD + likes
└── additional_tests.rs # Security, error handling, full user journey
```

### Container Setup Module

Create a `setup/mod.rs` that starts your service image and its dependencies
together, returning the containers so they can be stopped when the test ends:

```rust
// tests/integration/setup/mod.rs
use testcontainers::clients::Cli;
use testcontainers::core::{GenericContainer, WaitFor};
use testcontainers::images::generic::GenericImage;
use testcontainers::Container;
use testcontainers_modules::mongo;

/// Starts the service container along with its dependencies (MongoDB).
///
/// This function:
/// 1. Starts a MongoDB container
/// 2. Starts the rusta-example service container with the MongoDB connection string
/// 3. Waits for the service to print its startup log ("Listening on ...")
/// 4. Returns both containers so the caller can stop them when done
pub fn startServiceContainer() -> (Container<mongo::Mongo>, GenericContainer) {
    let docker = Cli::default();

    // Start MongoDB container
    let mongo_container = mongo::Mongo::default().start(&docker);

    // Get the MongoDB host port
    let mongo_port = mongo_container.get_host_port_ipv4(27017);
    let mongo_uri = format!("mongodb://localhost:{}", mongo_port);

    // Generate a unique database name for test isolation
    let db_name = format!("blog_test_{}", uuid::Uuid::new_v4().simple());

    // Build the service container with environment variables.
    // Use testcontainers' built-in wait strategy to block until the service
    // prints its startup log ("Listening on ..."), which is emitted by main.rs
    // once the HTTP server is bound and ready to accept connections.
    let service_image = GenericImage::new("rusta-example", "test")
        .with_exposed_port(3001u16)
        .with_env_var("MONGO_URI", mongo_uri)
        .with_env_var("MONGO_DB", db_name)
        .with_env_var("JWT_SECRET", "test_secret_for_integration")
        .with_env_var("JWT_EXPIRY_SECONDS", "3600")
        .with_env_var("SERVER_PORT", "0.0.0.0:3001")
        .with_wait_for(WaitFor::message_on_stdout("Listening on"));

    // Start the service container (blocks until the wait strategy resolves)
    let service_container = service_image.start(&docker);

    (mongo_container, service_container)
}
```

### Registering the Setup Module

Add the `setup` module to your integration test `mod.rs`:

```rust
// tests/integration/mod.rs
pub mod additional_tests;
pub mod auth_tests;
pub mod comment_tests;
pub mod post_tests;
pub mod setup;
```

### Writing Integration Tests

Call `startServiceContainer()` and exercise the HTTP API. The wait strategy
ensures the service is ready before the test runs:

```rust
// tests/integration/auth_tests.rs
use crate::setup::startServiceContainer;
use reqwest::Client;

#[tokio::test]
async fn test_register_success() {
    // Start containers (blocks until service is ready)
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Register a user
    use rusta_example::models::user::CreateUserDto;
    let dto = CreateUserDto {
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&dto)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Containers will be stopped when they go out of scope
}
```

### Why the Startup Wait Strategy?

The `with_wait_for(WaitFor::message_on_stdout("Listening on"))` call tells
testcontainers to **block** until the service prints its startup log message.
This is more reliable than HTTP polling because:

- It waits for an explicit readiness signal from the service itself
- It is faster (no retry delays)
- It eliminates race conditions between container startup and test execution

The service must print a recognizable message on stdout once it has bound
to its port. In `rusta-example`, `main.rs` prints `Listening on {addr}`.

### Container Lifecycle

Containers are automatically stopped when they go out of scope at the end
of each test function. testcontainers implements `Drop` on `Container<T>`,
which calls Docker to stop and remove the container.

### Prerequisite: Test Docker Image

Before running integration tests, build the Docker image that `startServiceContainer`
will reference:

```bash
docker build -t rusta-example:test -f rusta-example/Dockerfile .
```

If your tests reference a different image name, update the
`GenericImage::new("rusta-example", "test")` call in `startServiceContainer`
to match.

### Test Coverage Categories

The `rusta-example` integration suite covers:

| Category           | Examples                                                   |
| ------------------ | ---------------------------------------------------------- |
| **Auth**           | register, login, duplicate email, wrong password           |
| **Posts**          | list, create, get, update, delete, ownership               |
| **Comments**       | list, create, update, delete, like/unlike, ownership       |
| **Security**       | invalid JWT, tampered token, missing auth header           |
| **Error handling** | invalid JSON, method not allowed, 404 routes               |
| **Cross-cutting**  | full user journey, concurrent users, referential integrity |

### Full User Journey Example

```rust
#[tokio::test]
async fn test_full_user_journey() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container.get_host_port_ipv4(3001).await.unwrap();
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // 1. Register a user
    let token = register_user(&client, &base_url, "journey").await;

    // 2. Create a post
    let create_post_dto = CreatePostDto {
        title: "Journey Post".to_string(),
        body: "Body of journey post.".to_string(),
    };
    let response = client
        .post(format!("{}/posts/", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_post_dto)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    let post: serde_json::Value = response.json().await.unwrap();
    let post_id = post["id"].as_str().unwrap().to_string();

    // 3. Add a comment
    let comment_dto = CreateCommentDto {
        body: "Great post!".to_string(),
    };
    let response = client
        .post(format!("{}/posts/{}/comments", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&comment_dto)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // 4. Delete the post
    let response = client
        .delete(format!("{}/posts/{}", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
}
```

### Cargo.toml Setup

Add the required dev-dependencies:

```toml
[dev-dependencies]
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["mongo"] }
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
serde_json = "1"
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
│   ├── mod.rs              # Module declarations
│   ├── setup/
│   │   └── mod.rs          # startServiceContainer() helper
│   ├── auth_tests.rs       # /auth/register, /auth/login
│   ├── post_tests.rs       # /posts CRUD
│   ├── comment_tests.rs    # /posts/:id/comments CRUD + likes
│   └── additional_tests.rs # Security, error handling, full user journey
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
Each integration test should get its own database. Generate a unique database name
per test (e.g. `format!("blog_test_{}", uuid::Uuid::new_v4().simple())`) so
parallel tests don't interfere with each other.
:::

:::note[DI Makes Testing Easy]
The container pattern means you never need to mock the framework — just swap the service implementation.
Add `mockall` and `testcontainers` to dev-dependencies in Cargo.toml:

```toml
[dev-dependencies]
mockall = "0.12"
testcontainers = { version = "0.23", features = ["tokio"] }
testcontainers-modules = { version = "0.11", features = ["mongo"] }
reqwest = { version = "0.12", features = ["json"] }
```

:::

:::tip[Use Wait Strategies]
For end-to-end container tests, always use `wait_for` strategies to block until
the service is ready. Don't poll HTTP endpoints — it's slower and less reliable.

```rust
.with_wait_for(WaitFor::message_on_stdout("Listening on"))
```

:::
