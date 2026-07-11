use crate::test_harness::{login_user, register_user, TestContext};

/// Integration tests that run the full service in Docker containers
/// and exercise the HTTP API over the network.

#[tokio::test]
async fn test_register_success() {
    let ctx = TestContext::new().await;
    let _token = register_user(&ctx, "register_success").await;
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let ctx = TestContext::new().await;

    // First registration
    let _token = register_user(&ctx, "duplicate").await;

    // Second registration with same email should fail
    // register_user uses format!("test_{}@example.com", suffix) so we use the same email
    use ravix_example::models::user::CreateUserDto;
    let dto = CreateUserDto {
        username: "duplicate2".to_string(),
        email: "test_duplicate@example.com".to_string(), // Same email as first registration
        password: "password123".to_string(),
    };

    let response = ctx.post("/auth/register", &dto).await;
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login_success() {
    let ctx = TestContext::new().await;

    // Register first
    let _token = register_user(&ctx, "login_success").await;

    // Then login
    let token = login_user(&ctx, "test_login_success@example.com").await;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let ctx = TestContext::new().await;

    // Register first
    let _token = register_user(&ctx, "wrong_pass").await;

    // Login with wrong password
    use ravix_example::models::user::LoginDto;
    let dto = LoginDto {
        email: "test_wrong_pass@example.com".to_string(),
        password: "wrongpassword".to_string(),
    };

    let response = ctx.post("/auth/login", &dto).await;
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}
