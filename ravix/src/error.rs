use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response as AxumResponse},
};
use ravix_di::DiError;
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

/// Flexible error response that can be either a simple string or a structured object.
///
/// Use this for error responses in handlers. The `IntoResponse` implementation
/// automatically converts to JSON format.
///
/// # Variants
///
/// - `Message(String)` - Simple string error, renders as `{ "error": "message" }`
/// - `Object(Value)` - Structured error object, passed through as-is
///
/// # Example
/// ```
/// use ravix::ErrorResponse;
///
/// // Simple string error
/// let error = ErrorResponse::message("Not found");
///
/// // Structured error object
/// let error = ErrorResponse::object(serde_json::json!({
///     "code": "USER_NOT_FOUND",
///     "message": "User not found",
///     "details": { "user_id": 123 }
/// }));
/// ```
#[derive(Debug, Clone)]
pub enum ErrorResponse {
    /// Simple string error message
    Message(String),
    /// Structured error object (any JSON-serializable value)
    Object(serde_json::Value),
}

impl ErrorResponse {
    /// Create a simple string error response
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }

    /// Create a structured error object response
    pub fn object(obj: impl Serialize) -> Self {
        Self::Object(
            serde_json::to_value(obj)
                .unwrap_or_else(|_| json!({ "error": "serialization failed" })),
        )
    }

    /// Convert to JSON value for response
    pub fn into_json(self) -> serde_json::Value {
        match self {
            Self::Message(msg) => json!({ "error": msg }),
            Self::Object(obj) => obj,
        }
    }
}

impl From<String> for ErrorResponse {
    fn from(msg: String) -> Self {
        Self::Message(msg)
    }
}

impl From<&str> for ErrorResponse {
    fn from(msg: &str) -> Self {
        Self::Message(msg.to_string())
    }
}

impl From<serde_json::Value> for ErrorResponse {
    fn from(obj: serde_json::Value) -> Self {
        Self::Object(obj)
    }
}

#[derive(Debug, Error)]
pub enum FrameworkError {
    #[error("Dependency injection error: {0}")]
    InjectionError(String),

    #[error("Routing error: {0}")]
    RoutingError(String),

    #[error("Middleware error: {0}")]
    MiddlewareError(String),
}

impl From<DiError> for FrameworkError {
    fn from(err: DiError) -> Self {
        Self::InjectionError(err.to_string())
    }
}

impl IntoResponse for FrameworkError {
    fn into_response(self) -> AxumResponse {
        let (status, error_response) = match &self {
            Self::InjectionError(m) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::message(m))
            }
            Self::RoutingError(m) => (StatusCode::NOT_FOUND, ErrorResponse::message(m)),
            Self::MiddlewareError(m) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::message(m))
            }
        };
        (status, Json(error_response.into_json())).into_response()
    }
}
