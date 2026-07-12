use axum::response::IntoResponse;
use rusta::prelude::*;
use serde_json::json;

/// JWT guard middleware — validates Bearer token and inserts `Claims` as a
/// request extension so controllers can extract the authenticated user.
///
/// Usage: `#[middleware(jwt_guard)]` on controller handler methods.
pub async fn jwt_guard(
    State(container): State<ContainerRef>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response();
        }
    };

    // Resolve AuthService and verify token
    let auth_service = container.resolve::<std::sync::Arc<crate::services::AuthService>>();
    match auth_service.verify_token(&token).await {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired token" })),
        )
            .into_response(),
    }
}
