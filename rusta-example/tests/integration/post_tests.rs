use crate::setup::startServiceContainer;
use reqwest::Client;

/// Integration tests that run the full service in Docker containers
/// and exercise the HTTP API over the network.
///
/// The service container is started with a `LogWaitStrategy` that waits for
/// the startup log ("Listening on ..."), so by the time we get the port the
/// HTTP server is already accepting connections.

/// Helper to register a user and return the token
async fn register_user(client: &Client, base_url: &str, suffix: &str) -> String {
    use rusta_example::models::user::CreateUserDto;

    let dto = CreateUserDto {
        username: format!("postuser_{}", suffix),
        email: format!("post_{}@example.com", suffix),
        password: "password123".to_string(),
    };

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&dto)
        .send()
        .await
        .expect("POST /auth/register failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    let auth_response: serde_json::Value = response.json().await.expect("Failed to parse response");
    auth_response["token"]
        .as_str()
        .expect("No token in response")
        .to_string()
}

/// Helper to create a post and return its ID
async fn create_post(
    client: &Client,
    base_url: &str,
    token: &str,
    title: &str,
    body: &str,
) -> String {
    use rusta_example::models::post::CreatePostDto;

    let create_dto = CreatePostDto {
        title: title.to_string(),
        body: body.to_string(),
    };

    let response = client
        .post(format!("{}/posts/", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_dto)
        .send()
        .await
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    let post: serde_json::Value = response.json().await.expect("Failed to parse response");
    post["id"].as_str().expect("No id in response").to_string()
}

#[tokio::test]
async fn test_list_posts_empty() {
    // Start containers (blocks until service is ready)
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // List posts (should be empty initially)
    let response = client
        .get(format!("{}/posts/", base_url))
        .send()
        .await
        .expect("GET /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let posts: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert_eq!(posts.len(), 0);
}

#[tokio::test]
async fn test_create_post_unauthenticated() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Try to create a post without auth
    use rusta_example::models::post::CreatePostDto;
    let dto = CreatePostDto {
        title: "Test Post".to_string(),
        body: "This is a test post".to_string(),
    };

    let response = client
        .post(format!("{}/posts/", base_url))
        .json(&dto)
        .send()
        .await
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_post_authenticated() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Register a user
    let token = register_user(&client, &base_url, "create_post").await;

    // Create a post
    use rusta_example::models::post::CreatePostDto;
    let dto = CreatePostDto {
        title: "Test Post".to_string(),
        body: "This is a test post".to_string(),
    };

    let response = client
        .post(format!("{}/posts/", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&dto)
        .send()
        .await
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_posts() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "list_posts").await;

    // Create two posts
    for i in 0..2 {
        let _ = create_post(
            &client,
            &base_url,
            &token,
            &format!("Post {}", i),
            &format!("Body {}", i),
        )
        .await;
    }

    // List posts
    let response = client
        .get(format!("{}/posts/", base_url))
        .send()
        .await
        .expect("GET /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let posts: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert!(posts.len() >= 2);
}

#[tokio::test]
async fn test_get_post_by_id() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "get_post").await;
    let post_id = create_post(&client, &base_url, &token, "Get Me", "Body").await;

    // Get the post by id
    let response = client
        .get(format!("{}/posts/{}", base_url, post_id))
        .send()
        .await
        .expect("GET /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let post: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(post["id"], post_id);
    assert_eq!(post["title"], "Get Me");
}

#[tokio::test]
async fn test_get_post_not_found() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Try to get a non-existent post
    let response = client
        .get(format!("{}/posts/nonexistent_id_123", base_url))
        .send()
        .await
        .expect("GET /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_post_owner() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "update_owner").await;
    let post_id = create_post(&client, &base_url, &token, "Original", "Original body").await;

    // Update the post
    use rusta_example::models::post::UpdatePostDto;
    let update_dto = UpdatePostDto {
        title: Some("Updated Title".to_string()),
        body: Some("Updated body".to_string()),
    };

    let response = client
        .put(format!("{}/posts/{}", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_dto)
        .send()
        .await
        .expect("PUT /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_update_post_not_owner() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // User A creates a post
    let token_a = register_user(&client, &base_url, "update_a").await;
    let post_id = create_post(&client, &base_url, &token_a, "User A Post", "Body").await;

    // User B tries to update
    let token_b = register_user(&client, &base_url, "update_b").await;
    use rusta_example::models::post::UpdatePostDto;
    let update_dto = UpdatePostDto {
        title: Some("Hacked!".to_string()),
        body: None,
    };

    let response = client
        .put(format!("{}/posts/{}", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .json(&update_dto)
        .send()
        .await
        .expect("PUT /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_post_owner() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "delete").await;
    let post_id = create_post(&client, &base_url, &token, "To Delete", "Body").await;

    // Delete the post
    let response = client
        .delete(format!("{}/posts/{}", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("DELETE /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
}
