//! Integration test harness for running the blog-api service in Docker containers.
//!
//! This module provides utilities to:
//! - Build the Docker image for the service
//! - Start MongoDB and the service containers via testcontainers
//! - Wait for service readiness
//! - Make HTTP requests via reqwest

use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;
use testcontainers::{
    images::generic::GenericImage, runners::AsyncRunner, Container, RunnableImage,
};
use testcontainers_modules::mongo;

/// Docker image name for the test service
const TEST_IMAGE_NAME: &str = "ravix-example:test";

/// Global flag to ensure image is built only once per test run
static IMAGE_BUILT: Lazy<std::sync::Once> = Lazy::new(|| std::sync::Once::new());

/// Build the Docker image for the service if not already built.
/// This is called once per test run via the Lazy static.
fn ensure_image_built() {
    use std::process::Command;

    IMAGE_BUILT.call_once(|| {
        let output = Command::new("docker")
            .args([
                "build",
                "-t",
                TEST_IMAGE_NAME,
                "-f",
                "ravix-example/Dockerfile",
                ".",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output();

        match output {
            Ok(o) if o.status.success() => {
                eprintln!("Docker image {} built successfully", TEST_IMAGE_NAME);
            }
            Ok(o) => {
                panic!(
                    "Failed to build Docker image: {}",
                    String::from_utf8_lossy(&o.stderr)
                );
            }
            Err(e) => {
                panic!("Failed to execute docker build: {}", e);
            }
        }
    });
}

/// Test context holding the running containers and HTTP client
pub struct TestContext {
    /// The MongoDB container
    pub mongo_container: Container<mongo::Mongo>,
    /// The service container
    pub service_container: Container<GenericImage>,
    /// HTTP client for making requests
    pub client: Client,
    /// Base URL for the service
    pub base_url: String,
}

impl TestContext {
    /// Create a new test context with MongoDB and service containers running
    pub async fn new() -> Self {
        // Ensure the Docker image is built
        ensure_image_built();

        // Start MongoDB container
        let mongo_container = mongo::Mongo::default()
            .start()
            .await
            .expect("Failed to start MongoDB container");

        let mongo_host_port = mongo_container
            .get_host_port_ipv4(27017)
            .await
            .expect("Failed to get MongoDB port");
        let mongo_uri = format!("mongodb://localhost:{}", mongo_host_port);

        // Generate unique database name for test isolation
        let db_name = format!("blog_test_{}", uuid::Uuid::new_v4().simple());

        // Build the service container image with environment variables
        let service_image = GenericImage::new(TEST_IMAGE_NAME, "latest")
            .with_exposed_port(3001)
            .with_env_var(("MONGO_URI", mongo_uri.clone()))
            .with_env_var(("MONGO_DB", db_name.clone()))
            .with_env_var(("JWT_SECRET", "test_secret_for_integration"))
            .with_env_var(("JWT_EXPIRY_SECONDS", "3600"))
            .with_env_var(("SERVER_PORT", "0.0.0.0:3001"));

        let service_container = service_image
            .run()
            .await
            .expect("Failed to start service container");

        let service_port = service_container
            .get_host_port_ipv4(3001)
            .await
            .expect("Failed to get service port");

        let base_url = format!("http://localhost:{}", service_port);

        // Wait for service to be ready
        let client = Client::new();
        wait_for_service_ready(&client, &base_url).await;

        Self {
            mongo_container,
            service_container,
            client,
            base_url,
        }
    }

    /// Make a POST request to the service
    pub async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> reqwest::Response {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .json(body)
            .send()
            .await
            .expect("POST request failed")
    }

    /// Make a POST request with authorization
    pub async fn post_auth<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
        token: &str,
    ) -> reqwest::Response {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await
            .expect("POST request failed")
    }

    /// Make a POST request with authorization but no body (for endpoints like /like)
    pub async fn post_auth_empty(&self, path: &str, token: &str) -> reqwest::Response {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("POST request failed")
    }

    /// Make a GET request to the service
    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .expect("GET request failed")
    }

    /// Make a GET request with authorization
    pub async fn get_auth(&self, path: &str, token: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("GET request failed")
    }

    /// Make a PUT request with authorization
    pub async fn put_auth<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
        token: &str,
    ) -> reqwest::Response {
        self.client
            .put(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await
            .expect("PUT request failed")
    }

    /// Make a DELETE request with authorization
    pub async fn delete_auth(&self, path: &str, token: &str) -> reqwest::Response {
        self.client
            .delete(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("DELETE request failed")
    }
}

/// Wait for the service to be ready by polling the /posts endpoint
async fn wait_for_service_ready(client: &Client, base_url: &str) {
    let max_attempts = 30;
    let delay = Duration::from_millis(500);

    for attempt in 1..=max_attempts {
        match client.get(format!("{}/posts", base_url)).send().await {
            Ok(response) if response.status().is_success() || response.status().as_u16() == 404 => {
                eprintln!("Service ready after {} attempts", attempt);
                return;
            }
            _ => {
                if attempt == max_attempts {
                    panic!(
                        "Service did not become ready after {} attempts",
                        max_attempts
                    );
                }
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Helper to register a user and return the token
pub async fn register_user(ctx: &TestContext, suffix: &str) -> String {
    use ravix_example::models::user::CreateUserDto;

    let dto = CreateUserDto {
        username: format!("testuser_{}", suffix),
        email: format!("test_{}@example.com", suffix),
        password: "password123".to_string(),
    };

    let response = ctx.post("/auth/register", &dto).await;
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    let auth_response: serde_json::Value = response.json().await.expect("Failed to parse response");
    auth_response["token"]
        .as_str()
        .expect("No token in response")
        .to_string()
}

/// Helper to login and return the token
pub async fn login_user(ctx: &TestContext, email: &str) -> String {
    use ravix_example::models::user::LoginDto;

    let dto = LoginDto {
        email: email.to_string(),
        password: "password123".to_string(),
    };

    let response = ctx.post("/auth/login", &dto).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let auth_response: serde_json::Value = response.json().await.expect("Failed to parse response");
    auth_response["token"]
        .as_str()
        .expect("No token in response")
        .to_string()
}
