use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response as AxumResponse},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiError {
    #[error("Dependency injection error: {0}")]
    InjectionError(String),
}

impl IntoResponse for DiError {
    fn into_response(self) -> AxumResponse {
        let (status, message) = match &self {
            Self::InjectionError(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
