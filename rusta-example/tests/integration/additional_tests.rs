use crate::setup::startServiceContainer;
use reqwest::Client;

/// Additional integration tests covering security, error handling,
/// and cross-cutting user journeys.

/// Helper to register a user and return the token
async fn register_user(client: &Client, base_url: &str, suffix: &str) -> String {
    use rusta_example::models::user::CreateUserDto;

    let dto = CreateUserDto {
        username: format!("user_{}", suffix),
        email: format!("{}@example.com", suffix),
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

// ─────────────────────────────────────────────────────────────────────────────
// Security: JWT validation
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_invalid_token_rejected() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Use a totally bogus token
    let response = client
        .post(format!("{}/posts/", base_url))
        .header("Authorization", "Bearer this.is.not.a.real.token")
        .json(&serde_json::json!({"title": "x", "body": "y"}))
        .send()
        .await
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_tampered_token_rejected() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Get a valid token, then corrupt it
    let token = register_user(&client, &base_url, "tamper").await;

    // Corrupt the signature by changing the last 10 chars
    let mut chars: Vec<char> = token.chars().collect();
    let len = chars.len();
    for i in (len.saturating_sub(10))..len {
        chars[i] = if chars[i] == 'A' { 'B' } else { 'A' };
    }
    let tampered: String = chars.into_iter().collect();

    let response = client
        .post(format!("{}/posts/", base_url))
        .header("Authorization", format!("Bearer {}", tampered))
        .json(&serde_json::json!({"title": "x", "body": "y"}))
        .send()
        .await
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_missing_authorization_header() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // No Authorization header at all
    let response = client
        .post(format!("{}/posts/", base_url))
        .json(&serde_json::json!({"title": "x", "body": "y"}))
        .send()
        .await
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

// ─────────────────────────────────────────────────────────────────────────────
// Error handling: malformed requests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_invalid_json_body() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let response = client
        .post(format!("{}/auth/register", base_url))
        .header("Content-Type", "application/json")
        .body("{ this is not valid json }")
        .send()
        .await
        .expect("POST /auth/register failed");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_method_not_allowed() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // PATCH /posts is not a registered route
    let response = client
        .patch(format!("{}/posts/", base_url))
        .send()
        .await
        .expect("PATCH /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_not_found_route() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let response = client
        .get(format!("{}/nonexistent/route", base_url))
        .send()
        .await
        .expect("GET /nonexistent failed");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-cutting: full user journey
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_full_user_journey() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // 1. Register a user
    let token = register_user(&client, &base_url, "journey").await;
    assert!(!token.is_empty());

    // 2. Create a post
    use rusta_example::models::post::CreatePostDto;
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
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    let post: serde_json::Value = response.json().await.expect("Failed to parse response");
    let post_id = post["id"].as_str().expect("No post id").to_string();

    // 3. List posts and verify ours is there
    let response = client
        .get(format!("{}/posts/", base_url))
        .send()
        .await
        .expect("GET /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let posts: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert!(posts.iter().any(|p| p["id"] == post_id));

    // 4. Add a comment to the post
    use rusta_example::models::comment::CreateCommentDto;
    let create_comment_dto = CreateCommentDto {
        body: "Great post!".to_string(),
    };

    let response = client
        .post(format!("{}/posts/{}/comments", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_comment_dto)
        .send()
        .await
        .expect("POST /posts/{id}/comments failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    let comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    let comment_id = comment["id"].as_str().expect("No comment id").to_string();

    // 5. Like the comment
    let response = client
        .post(format!(
            "{}/posts/{}/comments/{}/like",
            base_url, post_id, comment_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("POST /like failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let liked_comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(liked_comment["like_count"], 1);

    // 6. Delete the post (cascades to comments)
    let response = client
        .delete(format!("{}/posts/{}", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("DELETE /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);

    // 7. Verify the post is gone
    let response = client
        .get(format!("{}/posts/{}", base_url, post_id))
        .send()
        .await
        .expect("GET /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_concurrent_users_interaction() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // User A creates a post
    let token_a = register_user(&client, &base_url, "alice").await;

    use rusta_example::models::post::CreatePostDto;
    let create_dto = CreatePostDto {
        title: "Alice's Post".to_string(),
        body: "Body by Alice.".to_string(),
    };

    let response = client
        .post(format!("{}/posts/", base_url))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&create_dto)
        .send()
        .await
        .expect("POST /posts failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    let post: serde_json::Value = response.json().await.expect("Failed to parse response");
    let post_id = post["id"].as_str().expect("No post id").to_string();

    // User B (Bob) registers and likes/comments on Alice's post
    let token_b = register_user(&client, &base_url, "bob").await;

    // Bob comments on Alice's post
    use rusta_example::models::comment::CreateCommentDto;
    let comment_dto = CreateCommentDto {
        body: "Nice post, Alice!".to_string(),
    };

    let response = client
        .post(format!("{}/posts/{}/comments", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .json(&comment_dto)
        .send()
        .await
        .expect("POST /comments failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Bob should NOT be able to delete Alice's post
    let response = client
        .delete(format!("{}/posts/{}", base_url, post_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .expect("DELETE /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);

    // The post should still exist
    let response = client
        .get(format!("{}/posts/{}", base_url, post_id))
        .send()
        .await
        .expect("GET /posts/{id} failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

// ─────────────────────────────────────────────────────────────────────────────
// Comments: referential integrity
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_comment_on_nonexistent_post() {
    let (_mongo_container, service_container) = startServiceContainer();

    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    let token = register_user(&client, &base_url, "orphan").await;

    // Try to comment on a post that doesn't exist
    use rusta_example::models::comment::CreateCommentDto;
    let dto = CreateCommentDto {
        body: "Comment on nothing".to_string(),
    };

    let response = client
        .post(format!(
            "{}/posts/nonexistent_post_id_xyz/comments",
            base_url
        ))
        .header("Authorization", format!("Bearer {}", token))
        .json(&dto)
        .send()
        .await
        .expect("POST /comments failed");

    // Should be NOT_FOUND because the post doesn't exist
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}
