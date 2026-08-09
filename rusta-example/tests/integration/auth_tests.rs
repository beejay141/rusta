use crate::setup::startServiceContainer;
use reqwest::Client;

/// Integration tests that run the full service in Docker containers
/// and exercise the HTTP API over the network.
///
/// The service container is started with a `LogWaitStrategy` that waits for
/// the startup log ("Listening on ..."), so by the time we get the port the
/// HTTP server is already accepting connections.

#[tokio::test]
async fn test_register_success() {
    // Start containers (blocks until service is ready)
    let (mongo_container, service_container) = startServiceContainer();

    // Get service port
    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Register user
    use rusta_example::models::user::CreateUserDto;
    let dto = CreateUserDto {
        username: "register_success".to_string(),
        email: "register_success@example.com".to_string(),
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

#[tokio::test]
async fn test_register_duplicate_email() {
    // Start containers (blocks until service is ready)
    let (mongo_container, service_container) = startServiceContainer();

    // Get service port
    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // First registration
    use rusta_example::models::user::CreateUserDto;
    let dto = CreateUserDto {
        username: "duplicate".to_string(),
        email: "test_duplicate@example.com".to_string(),
        password: "password123".to_string(),
    };

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&dto)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Second registration with same email should fail
    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&dto)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);

    // Containers will be stopped when they go out of scope
}

#[tokio::test]
async fn test_login_success() {
    // Start containers (blocks until service is ready)
    let (mongo_container, service_container) = startServiceContainer();

    // Get service port
    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Register first
    use rusta_example::models::user::CreateUserDto;
    let register_dto = CreateUserDto {
        username: "login_success".to_string(),
        email: "test_login_success@example.com".to_string(),
        password: "password123".to_string(),
    };

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&register_dto)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Then login
    use rusta_example::models::user::LoginDto;
    let login_dto = LoginDto {
        email: "test_login_success@example.com".to_string(),
        password: "password123".to_string(),
    };

    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&login_dto)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let auth_response: serde_json::Value = response.json().await.expect("Failed to parse response");
    let token = auth_response["token"]
        .as_str()
        .expect("No token in response");

    assert!(!token.is_empty());

    // Containers will be stopped when they go out of scope
}

#[tokio::test]
async fn test_login_wrong_password() {
    // Start containers (blocks until service is ready)
    let (mongo_container, service_container) = startServiceContainer();

    // Get service port
    let service_port = service_container
        .get_host_port_ipv4(3001)
        .await
        .expect("Failed to get service port");
    let base_url = format!("http://localhost:{}", service_port);
    let client = Client::new();

    // Register first
    use rusta_example::models::user::CreateUserDto;
    let register_dto = CreateUserDto {
        username: "wrong_pass".to_string(),
        email: "test_wrong_pass@example.com".to_string(),
        password: "password123".to_string(),
    };

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&register_dto)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Login with wrong password
    use rusta_example::models::user::LoginDto;
    let dto = LoginDto {
        email: "test_wrong_pass@example.com".to_string(),
        password: "wrongpassword".to_string(),
    };

    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&dto)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Containers will be stopped when they go out of scope
}
