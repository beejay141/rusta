use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response as AxumResponse},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrameworkError {
    #[error("Dependency injection error: {0}")]
    InjectionError(String),

    #[error("Routing error: {0}")]
    RoutingError(String),

    #[error("Middleware error: {0}")]
    MiddlewareError(String),
}

impl IntoResponse for FrameworkError {
    fn into_response(self) -> AxumResponse {
        let (status, message) = match &self {
            Self::InjectionError(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
            Self::RoutingError(m) => (StatusCode::NOT_FOUND, m.clone()),
            Self::MiddlewareError(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
