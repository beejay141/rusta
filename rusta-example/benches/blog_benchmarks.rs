use axum::{body::Body, http::Request};
use criterion::{criterion_group, criterion_main, Criterion};
use once_cell::sync::Lazy;
use rusta::Injectable;
use rusta_example::{db, models::post::CreatePostDto, models::user::CreateUserDto};
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::mongo;
use tokio::runtime::Runtime;
use tower::ServiceExt;

/// Shared MongoDB container for all benchmarks
static MONGO_URI: Lazy<String> = Lazy::new(|| {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = mongo::Mongo::default()
            .start()
            .await
            .expect("Failed to start MongoDB container");
        format!(
            "mongodb://localhost:{}",
            container.get_host_port_ipv4(27017).await.unwrap()
        )
    })
});

/// Helper to build a test app with real MongoDB
async fn build_test_app(mongo_uri: &str) -> axum::Router {
    use rusta::Container;
    use rusta_example::{
        config::AppConfig,
        repositories::{
            CommentRepository, MongoCommentRepository, MongoPostRepository, MongoUserRepository,
            PostRepository, UserRepository,
        },
        services::{AuthService, CommentService, PostService},
    };

    let mut container = Container::new();

    // Initialize MongoDB
    db::init_mongo(mongo_uri, "blog_bench").await;

    // Register config
    let config = AppConfig {
        mongo_uri: mongo_uri.to_string(),
        mongo_db: "blog_bench".to_string(),
        jwt_secret: "bench_secret".to_string(),
        jwt_expiry_seconds: 3600,
        server_port: "0.0.0.0:3001".to_string(),
    };
    container.register(Arc::new(config));

    // Register repositories
    let user_repo = MongoUserRepository::construct(&container) as Arc<dyn UserRepository>;
    container.register(user_repo);

    let post_repo = MongoPostRepository::construct(&container) as Arc<dyn PostRepository>;
    container.register(post_repo);

    let comment_repo = MongoCommentRepository::construct(&container) as Arc<dyn CommentRepository>;
    container.register(comment_repo);

    // Register services
    let auth_svc = AuthService::construct(&container);
    container.register(auth_svc);

    let post_svc = PostService::construct(&container);
    container.register(post_svc);

    let comment_svc = CommentService::construct(&container);
    container.register(comment_svc);

    // Build router
    let container_ref = std::sync::Arc::new(container);
    rusta::router::RouterBuilder::build(container_ref)
}

/// Helper to register a user and get a JWT token
async fn register_and_login(app: &axum::Router, suffix: &str) -> String {
    let register_dto = CreateUserDto {
        username: format!("bench_user_{}", suffix),
        email: format!("bench_{}@example.com", suffix),
        password: "password123".to_string(),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/register")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_dto).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    auth_response["token"].as_str().unwrap().to_string()
}

fn bench_post_list(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mongo_uri = MONGO_URI.clone();

    let app = rt.block_on(build_test_app(&mongo_uri));
    let token = rt.block_on(register_and_login(&app, "post_list"));

    // Seed posts
    rt.block_on(async {
        for i in 0..100 {
            let create_dto = CreatePostDto {
                title: format!("Post {}", i),
                body: format!("Body {}", i),
            };
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/posts")
                        .method("POST")
                        .header("content-type", "application/json")
                        .header("authorization", format!("Bearer {}", token))
                        .body(Body::from(serde_json::to_vec(&create_dto).unwrap()))
                        .unwrap(),
                )
                .await;
        }
    });

    c.bench_function("post_list", |b| {
        b.to_async(&rt).iter(|| async {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/posts")
                        .method("GET")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
        })
    });
}

fn bench_post_create(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mongo_uri = MONGO_URI.clone();

    let app = rt.block_on(build_test_app(&mongo_uri));
    let token = rt.block_on(register_and_login(&app, "post_create"));

    c.bench_function("post_create", |b| {
        b.to_async(&rt).iter(|| async {
            let create_dto = CreatePostDto {
                title: "Benchmark Post".to_string(),
                body: "Benchmark Body".to_string(),
            };
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/posts")
                        .method("POST")
                        .header("content-type", "application/json")
                        .header("authorization", format!("Bearer {}", token))
                        .body(Body::from(serde_json::to_vec(&create_dto).unwrap()))
                        .unwrap(),
                )
                .await;
        })
    });
}

fn bench_auth_login(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mongo_uri = MONGO_URI.clone();

    let app = rt.block_on(build_test_app(&mongo_uri));

    // Register user
    rt.block_on(async {
        let register_dto = CreateUserDto {
            username: "bench_login_user".to_string(),
            email: "bench_login@example.com".to_string(),
            password: "password123".to_string(),
        };
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/register")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&register_dto).unwrap()))
                    .unwrap(),
            )
            .await;
    });

    c.bench_function("auth_login", |b| {
        b.to_async(&rt).iter(|| async {
            let login_dto = rusta_example::models::user::LoginDto {
                email: "bench_login@example.com".to_string(),
                password: "password123".to_string(),
            };
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/auth/login")
                        .method("POST")
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_vec(&login_dto).unwrap()))
                        .unwrap(),
                )
                .await;
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_post_list, bench_post_create, bench_auth_login
}

criterion_main!(benches);
