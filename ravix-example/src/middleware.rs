use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response, Json},
};
use serde_json::json;

/// Bearer-token auth guard for use with `#[middleware(auth_guard)]`.
///
/// Returns `401 Unauthorized` when the `Authorization` header is absent or
/// does not start with `"Bearer "`.
///
/// For production use, replace the presence check with real JWT validation.
pub async fn auth_guard(request: Request<Body>, next: Next) -> Response {
    let has_bearer = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer "))
        .unwrap_or(false);

    if has_bearer {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Unauthorized: missing or invalid Bearer token" })),
        )
            .into_response()
    }
}
