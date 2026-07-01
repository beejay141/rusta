use axum::{
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse, Json},
};
use serde::Serialize;
use serde_json::json;

/// Convenience helpers for building HTTP responses.
///
/// Every method returns `axum::response::Response` (re-exported as
/// [`AxumResponse`]) so they compose naturally with axum handlers.
pub struct Response;

impl Response {
    /// 200 OK with a JSON-serialised body.
    pub fn json<T: Serialize>(data: T) -> AxumResponse {
        Json(data).into_response()
    }

    /// 200 OK with an empty body.
    pub fn ok() -> AxumResponse {
        StatusCode::OK.into_response()
    }

    /// 201 Created with a JSON-serialised body.
    pub fn created<T: Serialize>(data: T) -> AxumResponse {
        (StatusCode::CREATED, Json(data)).into_response()
    }

    /// Arbitrary status code with an empty body.
    pub fn status(code: u16) -> AxumResponse {
        StatusCode::from_u16(code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response()
    }

    /// 404 Not Found with a JSON error message.
    pub fn not_found(message: &str) -> AxumResponse {
        (StatusCode::NOT_FOUND, Json(json!({ "error": message }))).into_response()
    }

    /// 401 Unauthorized with a JSON error message.
    pub fn unauthorized(message: &str) -> AxumResponse {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": message }))).into_response()
    }

    /// 400 Bad Request with a JSON error message.
    pub fn bad_request(message: &str) -> AxumResponse {
        (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response()
    }

    /// 500 Internal Server Error with a JSON error message.
    pub fn internal_error(message: &str) -> AxumResponse {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": message }))).into_response()
    }
}
