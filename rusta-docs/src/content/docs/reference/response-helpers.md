---
title: Response Helpers
description: Complete reference for Http response methods and error handling.
sidebar:
  order: 1
---

# Response Helpers

:::tip[Quick Reference]
All `Http` methods return `Response` — use directly in handler return types or wrap in `Ok()` for `Result<Response, AppError>`.
:::

## Http Methods

| Method                    | Status | Description                      |
| ------------------------- | ------ | -------------------------------- |
| `json(data)`              | 200    | JSON response with body          |
| `created(data)`           | 201    | Created with JSON body           |
| `ok()`                    | 200    | Empty OK response                |
| `no_content()`            | 204    | No Content response              |
| `status(code)`            | custom | Empty response with status code  |
| `with_status(code, body)` | custom | JSON response with custom status |

## Error Methods

| Method                     | Status | Description                          |
| -------------------------- | ------ | ------------------------------------ |
| `error(status, error)`     | custom | Flexible error with any status       |
| `not_found(msg)`           | 404    | Not Found with string error          |
| `not_found_with(obj)`      | 404    | Not Found with structured error      |
| `unauthorized(msg)`        | 401    | Unauthorized with string error       |
| `unauthorized_with(obj)`   | 401    | Unauthorized with structured error   |
| `bad_request(msg)`         | 400    | Bad Request with string error        |
| `bad_request_with(obj)`    | 400    | Bad Request with structured error    |
| `forbidden(msg)`           | 403    | Forbidden with string error          |
| `forbidden_with(obj)`      | 403    | Forbidden with structured error      |
| `internal_error(msg)`      | 500    | Internal Server Error                |
| `internal_error_with(obj)` | 500    | Internal Error with structured error |

## ErrorResponse Variants

```rust
pub enum ErrorResponse {
    Message(String),       // Simple string: { "error": "message" }
    Object(Value),         // Structured object: passed through as-is
}
```

## ErrorObject Wrapper

For structured error responses:

```rust
use rusta::{Http, ErrorObject};

#[derive(Serialize)]
struct ValidationError {
    code: &'static str,
    fields: Vec<&'static str>,
}

Http::bad_request(ErrorObject(ValidationError {
    code: "VALIDATION_ERROR",
    fields: vec!["email", "password"],
}));
```

## Examples

### Success Responses

```rust
// 200 OK with data
Http::json(user)

// 201 Created
Http::created(new_user)

// 204 No Content
Http::no_content()

// Custom status
Http::status(202)
Http::with_status(202, json!({ "status": "processing" }))
```

### Error Responses

```rust
// Simple errors
Http::not_found("User not found")
Http::unauthorized("Invalid token")
Http::bad_request("Missing field")
Http::forbidden("Access denied")
Http::internal_error("Database error")

// Structured errors
Http::error(400, ErrorObject(ValidationError {
    code: "INVALID_INPUT",
    fields: vec!["email"],
}))
```

## Custom Error Types

Implement `IntoResponse` for custom error handling:

```rust
use axum::{response::{IntoResponse, Json}, http::StatusCode};
use rusta::ErrorResponse;

pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    ValidationError(Vec<String>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorResponse::message(msg)),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, ErrorResponse::message(msg)),
            AppError::ValidationError(fields) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::object(json!({
                    "code": "VALIDATION_ERROR",
                    "fields": fields
                }))
            ),
        };
        (status, Json(error.into_json())).into_response()
    }
}
```
