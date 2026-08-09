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
        username: format!("commentuser_{}", suffix),
        email: format!("comment_{}@example.com", suffix),
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

/// Helper to create a comment and return its ID
async fn create_comment(
    client: &Client,
    base_url: &str,
    token: &str,
    post_id: &str,
    body: &str,
) -> String {
    use rusta_example::models::comment::CreateCommentDto;

    let create_dto = CreateCommentDto {
        body: body.to_string(),
    };

    let response = client
        .post(format!("{}/posts/{}/comments", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_dto)
        .send()
        .await
        .expect("POST /posts/{post_id}/comments failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    let comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    comment["id"]
        .as_str()
        .expect("No id in response")
        .to_string()
}

#[tokio::test]
async fn test_list_comments_empty() {
    // Start containers (blocks until service is ready)
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "list_empty").await;
    let post_id = create_post(&client, &base_url, &token, "Post for Comments", "Body").await;

    // List comments (should be empty)
    let response = client
        .get(format!("{}/posts/{}/comments", base_url, post_id))
        .send()
        .await
        .expect("GET /posts/{post_id}/comments failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let comments: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert_eq!(comments.len(), 0);
}

#[tokio::test]
async fn test_create_comment_unauthenticated() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "create_unauth").await;
    let post_id = create_post(&client, &base_url, &token, "Auth Post", "Body").await;

    // Try to create a comment without auth
    use rusta_example::models::comment::CreateCommentDto;
    let dto = CreateCommentDto {
        body: "Unauthorized comment".to_string(),
    };

    let response = client
        .post(format!("{}/posts/{}/comments", base_url, post_id))
        .json(&dto)
        .send()
        .await
        .expect("POST /posts/{post_id}/comments failed");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_comment_authenticated() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "create_auth").await;
    let post_id = create_post(&client, &base_url, &token, "Comment Post", "Body").await;

    // Create a comment
    let _comment_id =
        create_comment(&client, &base_url, &token, &post_id, "My first comment").await;
}

#[tokio::test]
async fn test_create_comment_validation_error() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "validation").await;
    let post_id = create_post(&client, &base_url, &token, "Validation Post", "Body").await;

    // Try to create a comment with empty body
    use rusta_example::models::comment::CreateCommentDto;
    let dto = CreateCommentDto {
        body: "".to_string(),
    };

    let response = client
        .post(format!("{}/posts/{}/comments", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&dto)
        .send()
        .await
        .expect("POST /posts/{post_id}/comments failed");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_comments() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "list_comments").await;
    let post_id = create_post(&client, &base_url, &token, "List Post", "Body").await;

    // Create two comments
    for i in 0..2 {
        let _ = create_comment(
            &client,
            &base_url,
            &token,
            &post_id,
            &format!("Comment {}", i),
        )
        .await;
    }

    // List comments
    let response = client
        .get(format!("{}/posts/{}/comments", base_url, post_id))
        .send()
        .await
        .expect("GET /posts/{post_id}/comments failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let comments: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert!(comments.len() >= 2);
}

#[tokio::test]
async fn test_update_comment_owner() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "update_owner").await;
    let post_id = create_post(&client, &base_url, &token, "Update Post", "Body").await;
    let comment_id = create_comment(&client, &base_url, &token, &post_id, "Original comment").await;

    // Update the comment
    use rusta_example::models::comment::UpdateCommentDto;
    let update_dto = UpdateCommentDto {
        body: "Updated comment".to_string(),
    };

    let response = client
        .put(format!(
            "{}/posts/{}/comments/{}",
            base_url, post_id, comment_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_dto)
        .send()
        .await
        .expect("PUT /posts/{post_id}/comments/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_update_comment_not_owner() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // User A creates a post and comment
    let token_a = register_user(&client, &base_url, "update_a").await;
    let post_id = create_post(&client, &base_url, &token_a, "User A Post", "Body").await;
    let comment_id = create_comment(&client, &base_url, &token_a, &post_id, "A's comment").await;

    // User B tries to update
    let token_b = register_user(&client, &base_url, "update_b").await;
    use rusta_example::models::comment::UpdateCommentDto;
    let update_dto = UpdateCommentDto {
        body: "Hacked!".to_string(),
    };

    let response = client
        .put(format!(
            "{}/posts/{}/comments/{}",
            base_url, post_id, comment_id
        ))
        .header("Authorization", format!("Bearer {}", token_b))
        .json(&update_dto)
        .send()
        .await
        .expect("PUT /posts/{post_id}/comments/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_comment_owner() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "delete").await;
    let post_id = create_post(&client, &base_url, &token, "Delete Post", "Body").await;
    let comment_id = create_comment(&client, &base_url, &token, &post_id, "To delete").await;

    // Delete the comment
    let response = client
        .delete(format!(
            "{}/posts/{}/comments/{}",
            base_url, post_id, comment_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("DELETE /posts/{post_id}/comments/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_like_comment() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "like").await;
    let post_id = create_post(&client, &base_url, &token, "Like Post", "Body").await;
    let comment_id = create_comment(&client, &base_url, &token, &post_id, "Like me").await;

    // Like the comment
    let response = client
        .post(format!(
            "{}/posts/{}/comments/{}/like",
            base_url, post_id, comment_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("POST /posts/{post_id}/comments/{id}/like failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(comment["like_count"], 1);
}

#[tokio::test]
async fn test_unlike_comment() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "unlike").await;
    let post_id = create_post(&client, &base_url, &token, "Unlike Post", "Body").await;
    let comment_id = create_comment(&client, &base_url, &token, &post_id, "Unlike me").await;

    // Like then unlike
    let _ = client
        .post(format!(
            "{}/posts/{}/comments/{}/like",
            base_url, post_id, comment_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("POST /like failed");

    let response = client
        .delete(format!(
            "{}/posts/{}/comments/{}/like",
            base_url, post_id, comment_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("DELETE /like failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(comment["like_count"], 0);
}

#[tokio::test]
async fn test_like_comment_unauthenticated() {
    // Start containers
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "like_unauth").await;
    let post_id = create_post(&client, &base_url, &token, "Like Post", "Body").await;
    let comment_id = create_comment(&client, &base_url, &token, &post_id, "Like me").await;

    // Try to like without auth
    let response = client
        .post(format!(
            "{}/posts/{}/comments/{}/like",
            base_url, post_id, comment_id
        ))
        .send()
        .await
        .expect("POST /like failed");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}
