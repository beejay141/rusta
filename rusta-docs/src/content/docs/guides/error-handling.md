---
title: Error Handling
description: Handle errors gracefully with structured responses, custom error types, and middleware.
sidebar:
  order: 4
---

# Error Handling

Rusta provides flexible error handling through `Http` response helpers, `ErrorResponse` variants, and custom `IntoResponse` implementations.

:::tip[Error Philosophy]
Prefer `Result<Response, AppError>` in handlers. This makes error types explicit, enables `?` operator, and keeps success paths clean.
:::

## Built-in Error Responses

### Simple String Errors

```rust
use rusta::Http;

#[get("/users/:id")]
pub async fn get_user(&self, Path(id): Path<String>) -> Response {
    match self.svc.find(&id).await {
        Some(user) => Http::json(user),
        None => Http::not_found("User not found"),
    }
}
```

| Method                      | Status | Use Case                      |
| --------------------------- | ------ | ----------------------------- |
| `Http::not_found(msg)`      | 404    | Resource doesn't exist        |
| `Http::unauthorized(msg)`   | 401    | Missing/invalid auth          |
| `Http::forbidden(msg)`      | 403    | Authenticated but not allowed |
| `Http::bad_request(msg)`    | 400    | Invalid input                 |
| `Http::internal_error(msg)` | 500    | Server error                  |

### Structured Errors with `ErrorObject`

```rust
use rusta::{Http, ErrorObject};
use serde_json::json;

Http::bad_request(ErrorObject(json!({
    "code": "VALIDATION_ERROR",
    "fields": ["email", "password"],
})))
```

Response:

```json
{
  "code": "VALIDATION_ERROR",
  "fields": ["email", "password"]
}
```

### Flexible Errors with `Http::error`

```rust
Http::error(422, ErrorObject(UnprocessableEntity {
    reason: "Email already registered",
    field: "email",
}))
```

## Custom Error Types

Implement `IntoResponse` for domain-specific errors:

```rust
use axum::{response::{IntoResponse, Json}, http::StatusCode};
use rusta::ErrorResponse;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation failed: {0:?}")]
    Validation(Vec<String>),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::Message(msg),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::Message(msg),
            ),
            AppError::Validation(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponse::Object(json!({
                    "code": "VALIDATION_ERROR",
                    "fields": fields,
                })),
            ),
            AppError::Database(e) => {
                tracing::error!(%e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::Message("Internal server error".into()),
                )
            },
        };

        (status, Json(error.into_json())).into_response()
    }
}
```

## Using Custom Errors in Handlers

```rust
#[get("/users/:id")]
pub async fn get_user(&self, Path(id): Path<String>) -> Result<Response, AppError> {
    let user = self.svc.find(&id).await
        .ok_or_else(|| AppError::NotFound(id))?;
    Ok(Http::json(user))
}

#[post("/users")]
pub async fn create_user(&self, Json(body): Json<CreateUserDto>) -> Result<Response, AppError> {
    self.svc.validate(&body)?;
    let user = self.svc.create(body).await?;
    Ok(Http::created(user))
}
```

## Validation Errors

Use `validator` crate for struct validation:

```rust
use validator::Validate;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(length(min = 1, max = 50))]
    pub name: String,
}

impl CreateUserDto {
    pub fn validate(&self) -> Result<(), AppError> {
        self.validate()
            .map_err(|e| AppError::Validation(
                e.field_errors()
                    .into_iter()
                    .flat_map(|(field, errors)| {
                        errors.into_iter().map(move |err| {
                            format!("{}: {}", field, err.message.unwrap_or_default())
                        })
                    })
                    .collect()
            ))
    }
}
```

## Error Response Format Consistency

All error responses follow this structure:

```json
// Simple error
{ "error": "User not found" }

// Structured error
{
  "code": "VALIDATION_ERROR",
  "fields": ["email", "password"]
}
```

:::caution[Consistency]
Always use `ErrorResponse::Message` for simple errors and `ErrorResponse::Object` for structured errors. This keeps your API predictable for clients.
:::

## Logging Errors

Errors are automatically logged when using the logger middleware. For custom logging:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::Database(e) => tracing::error!(%e, "Database error"),
            AppError::Validation(fields) => tracing::warn!(?fields, "Validation failed"),
            _ => tracing::info!(%self, "Handled error"),
        }
        // ... rest of implementation
    }
}
```

## Best Practices

1. **Return `Result<Response, AppError>`** — Makes errors explicit, enables `?`
2. **Use `thiserror`** — Derive `Error` with minimal boilerplate
3. **Log at the right level** — `error` for bugs, `warn` for client errors, `info` for expected flows
4. **Don't leak internals** — Map database errors to generic "Internal server error"
5. **Validate early** — Use DTO validation before business logic

:::tip[Error Codes]
Use consistent error codes across your API: `NOT_FOUND`, `UNAUTHORIZED`, `VALIDATION_ERROR`, `INTERNAL_ERROR`, `CONFLICT`, `RATE_LIMITED`.
:::

:::caution[Don't Leak Internals]
Never expose database errors, stack traces, or internal details in production responses. Log them internally, return generic messages to clients.
:::

:::note[Result vs Response]
Return `Result<Response, AppError>` from handlers for clean error propagation. The `IntoResponse` impl handles conversion automatically.
:::
