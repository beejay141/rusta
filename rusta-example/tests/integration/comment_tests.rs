use crate::test_harness::{TestContext, register_user};
use ravix_example::models::post::CreatePostDto;
use ravix_example::models::comment::{CreateCommentDto, UpdateCommentDto};

/// Integration tests that run the full service in Docker containers
/// and exercise the HTTP API over the network.

/// Helper to create a post and return its ID
async fn create_post(ctx: &TestContext, token: &str) -> String {
    let create_dto = CreatePostDto {
        title: "Test Post".to_string(),
        body: "Test Body".to_string(),
    };

    let response = ctx.post_auth("/posts", &create_dto, token).await;
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    let post: serde_json::Value = response.json().await.expect("Failed to parse response");
    post["id"].as_str().expect("No id in response").to_string()
}

/// Helper to create a comment and return its ID
async fn create_comment(ctx: &TestContext, token: &str, post_id: &str) -> String {
    let create_dto = CreateCommentDto {
        body: "This is a comment".to_string(),
    };

    let response = ctx.post_auth(&format!("/posts/{}/comments", post_id), &create_dto, token).await;
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    let comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    comment["id"].as_str().expect("No id in response").to_string()
}

#[tokio::test]
async fn test_create_comment() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "comment").await;
    let post_id = create_post(&ctx, &token).await;

    let create_dto = CreateCommentDto {
        body: "This is a comment".to_string(),
    };

    let response = ctx.post_auth(&format!("/posts/{}/comments", post_id), &create_dto, &token).await;
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[tokio::test]
async fn test_like_comment() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "like").await;
    let post_id = create_post(&ctx, &token).await;
    let comment_id = create_comment(&ctx, &token, &post_id).await;

    // Like the comment
    let response = ctx.post_auth_empty(&format!("/posts/{}/comments/{}/like", post_id, comment_id), &token).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let updated_comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_comment["like_count"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn test_unlike_comment() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "unlike").await;
    let post_id = create_post(&ctx, &token).await;
    let comment_id = create_comment(&ctx, &token, &post_id).await;

    // Like then unlike
    let _ = ctx.post_auth_empty(&format!("/posts/{}/comments/{}/like", post_id, comment_id), &token).await;

    let response = ctx.delete_auth(&format!("/posts/{}/comments/{}/like", post_id, comment_id), &token).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let updated_comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_comment["like_count"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn test_double_like_idempotent() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "double_like").await;
    let post_id = create_post(&ctx, &token).await;

    // Create a comment
    let comment_id = create_comment(&ctx, &token, &post_id).await;

    // Like twice
    let _ = ctx.post_auth_empty(&format!("/posts/{}/comments/{}/like", post_id, comment_id), &token).await;

    let response = ctx.post_auth_empty(&format!("/posts/{}/comments/{}/like", post_id, comment_id), &token).await;

    let updated_comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    // $addToSet semantics: double like should still be count of 1
    assert_eq!(updated_comment["like_count"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn test_edit_comment_not_owner() {
    let ctx = TestContext::new().await;

    // User A creates a post and comment
    let token_a = register_user(&ctx, "edit_a").await;
    let post_id = create_post(&ctx, &token_a).await;

    let create_dto = CreateCommentDto {
        body: "User A comment".to_string(),
    };

    let response = ctx.post_auth(&format!("/posts/{}/comments", post_id), &create_dto, &token_a).await;
    let comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    let comment_id = comment["id"].as_str().expect("No id in response");

    // User B tries to edit
    let token_b = register_user(&ctx, "edit_b").await;
    let update_dto = UpdateCommentDto {
        body: "Hacked!".to_string(),
    };

    let response = ctx.put_auth(&format!("/posts/{}/comments/{}", post_id, comment_id), &update_dto, &token_b).await;
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_comment_owner() {
    let ctx = TestContext::new().await;
    let token = register_user(&ctx, "delete_comment").await;
    let post_id = create_post(&ctx, &token).await;

    // Create a comment
    let create_dto = CreateCommentDto {
        body: "To delete".to_string(),
    };

    let response = ctx.post_auth(&format!("/posts/{}/comments", post_id), &create_dto, &token).await;
    let comment: serde_json::Value = response.json().await.expect("Failed to parse response");
    let comment_id = comment["id"].as_str().expect("No id in response");

    // Delete the comment
    let response = ctx.delete_auth(&format!("/posts/{}/comments/{}", post_id, comment_id), &token).await;
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
}