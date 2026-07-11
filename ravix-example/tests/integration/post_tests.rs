use crate::test_harness::{TestContext, register_user};
use ravix_example::models::post::{CreatePostDto, UpdatePostDto};

/// Integration tests that run the full service in Docker containers
/// and exercise the HTTP API over the network.

/// Helper to create a post and return its ID
async fn create_post(ctx: &TestContext, token: &str, title: &str, body: &str) -> String {
    let create_dto = CreatePostDto {
        title: title.to_string(),
        body: body.to_string(),
    };

    let response = ctx.post_auth("/posts", &create_dto, token).await;
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    let post: serde_json::Value = response.json().await.expect("Failed to parse response");
    post["id"].as_str().expect("No id in response").to_string()
}

#[tokio::test]
async fn test_create_post_unauthenticated() {
    let ctx = TestContext::new().await;

    let create_dto = CreatePostDto {
        title: "Test Post".to_string(),
        body: "This is a test post".to_string(),
    };

    let response = ctx.post("/posts", &create_dto).await;
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_post_authenticated() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "create_post").await;

    let create_dto = CreatePostDto {
        title: "Test Post".to_string(),
        body: "This is a test post".to_string(),
    };

    let response = ctx.post_auth("/posts", &create_dto, &token).await;
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_posts() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "list_posts").await;

    // Create two posts
    for i in 0..2 {
        let _ = create_post(&ctx, &token, &format!("Post {}", i), &format!("Body {}", i)).await;
    }

    // List posts
    let response = ctx.get("/posts").await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let posts: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(posts.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_post_not_owner() {
    let ctx = TestContext::new().await;

    // User A creates a post
    let token_a = register_user(&ctx, "update_a").await;
    let post_id = create_post(&ctx, &token_a, "User A Post", "Body").await;

    // User B tries to update
    let token_b = register_user(&ctx, "update_b").await;
    let update_dto = UpdatePostDto {
        title: Some("Hacked!".to_string()),
        body: None,
    };

    let response = ctx.put_auth(&format!("/posts/{}", post_id), &update_dto, &token_b).await;
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_post_owner() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "delete").await;

    // Create a post
    let post_id = create_post(&ctx, &token, "To Delete", "Body").await;

    // Delete the post
    let response = ctx.delete_auth(&format!("/posts/{}", post_id), &token).await;
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
}