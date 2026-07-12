use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rusta::ErrorResponse;
use bson::document::ValueAccessError;

/// Application-wide error type returned by service methods.
///
/// Controllers can `?`-propagate this error because it implements
/// `IntoResponse`.
#[derive(Debug, Clone)]
pub enum AppError {
    NotFound(String),
    NotFoundWith(serde_json::Value),
    Unauthorized(String),
    UnauthorizedWith(serde_json::Value),
    Forbidden(String),
    ForbiddenWith(serde_json::Value),
    BadRequest(String),
    BadRequestWith(serde_json::Value),
    Conflict(String),
    ConflictWith(serde_json::Value),
    DatabaseError(String),
    InternalError(String),
    InternalErrorWith(serde_json::Value),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::NotFoundWith(obj) => write!(f, "Not found: {}", obj),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::UnauthorizedWith(obj) => write!(f, "Unauthorized: {}", obj),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::ForbiddenWith(obj) => write!(f, "Forbidden: {}", obj),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::BadRequestWith(obj) => write!(f, "Bad request: {}", obj),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::ConflictWith(obj) => write!(f, "Conflict: {}", obj),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::InternalErrorWith(obj) => write!(f, "Internal error: {}", obj),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_response) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorResponse::message(msg)),
            AppError::NotFoundWith(obj) => (StatusCode::NOT_FOUND, ErrorResponse::object(obj)),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, ErrorResponse::message(msg)),
            AppError::UnauthorizedWith(obj) => (StatusCode::UNAUTHORIZED, ErrorResponse::object(obj)),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, ErrorResponse::message(msg)),
            AppError::ForbiddenWith(obj) => (StatusCode::FORBIDDEN, ErrorResponse::object(obj)),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ErrorResponse::message(msg)),
            AppError::BadRequestWith(obj) => (StatusCode::BAD_REQUEST, ErrorResponse::object(obj)),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, ErrorResponse::message(msg)),
            AppError::ConflictWith(obj) => (StatusCode::CONFLICT, ErrorResponse::object(obj)),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::message(msg)),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::message(msg)),
            AppError::InternalErrorWith(obj) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::object(obj)),
        };
        (status, Json(error_response.into_json())).into_response()
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<ValueAccessError> for AppError {
    fn from(e: ValueAccessError) -> Self {
        AppError::InternalError(format!("BSON field access error: {}", e))
    }
}
