// Re-export axum extractors and HTTP types under the rusta namespace so
// users only need to import from `rusta`.
pub use axum::extract::Json;
pub use axum::extract::{Extension, Path, Query, State};
pub use axum::http::request::Parts;
pub use axum::http::{HeaderMap, Method, StatusCode};

use axum::{
    extract::{FromRequest, Json as AxumJson, Request},
    http::StatusCode as AxumStatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use validator::{Validate, ValidationErrors};

/// Trait for customizing the HTTP response produced when validation fails.
///
/// Implement this on a zero-sized type and use it as the second type parameter
/// of [`ValidatedJson`] to control the exact shape of the error payload.
///
/// # Example
///
/// ```rust,ignore
/// use rusta::{ValidationErrorFormatter, ValidatedJson};
/// use validator::ValidationErrors;
/// use axum::response::Response;
///
/// #[derive(Default)]
/// struct ApiProblemFormatter;
///
/// impl ValidationErrorFormatter for ApiProblemFormatter {
///     fn format(errors: ValidationErrors) -> Response {
///         rusta::Http::bad_request_with(serde_json::json!({
///             "type": "https://api.example.com/errors/validation",
///             "title": "Validation failed",
///             "errors": errors,
///         }))
///     }
/// }
///
/// async fn handler(ValidatedJson(body, ..): ValidatedJson<MyDto, ApiProblemFormatter>) {}
/// ```
pub trait ValidationErrorFormatter: Send + Sync + 'static {
    /// Convert validation errors into an HTTP response.
    fn format(errors: ValidationErrors) -> Response;
}

/// Default validation error formatter.
///
/// Returns `400 Bad Request` with:
/// ```json
/// { "error": "Validation error", "details": { ... } }
/// ```
#[derive(Default, Clone, Copy, Debug)]
pub struct DefaultValidationErrorFormatter;

impl ValidationErrorFormatter for DefaultValidationErrorFormatter {
    fn format(errors: ValidationErrors) -> Response {
        #[derive(Serialize)]
        struct Body {
            error: &'static str,
            details: ValidationErrors,
        }

        (
            AxumStatusCode::BAD_REQUEST,
            AxumJson(Body {
                error: "Validation error",
                details: errors,
            }),
        )
            .into_response()
    }
}

/// A validated JSON body extractor.
///
/// Behaves like [`Json`], but automatically runs [`validator::Validate`] after
/// deserialization. If validation fails, the request is rejected with
/// `400 Bad Request` and a structured JSON error body.
///
/// The error body shape is controlled by the `F` type parameter. Use the
/// default [`DefaultValidationErrorFormatter`] for the built-in format, or
/// implement [`ValidationErrorFormatter`] for a custom response shape.
///
/// # Example
///
/// ```rust,ignore
/// use rusta::{Http, ValidatedJson};
///
/// #[derive(serde::Deserialize, validator::Validate)]
/// struct CreateUserDto {
///     #[validate(email)]
///     email: String,
/// }
///
/// async fn handler(ValidatedJson(body, ..): ValidatedJson<CreateUserDto>) -> rusta::Response {
///     Http::json(body)
/// }
/// ```
pub struct ValidatedJson<T, F = DefaultValidationErrorFormatter>(pub T, pub std::marker::PhantomData<F>);

#[axum::async_trait]
impl<T, F, S> FromRequest<S> for ValidatedJson<T, F>
where
    T: DeserializeOwned + Validate,
    F: ValidationErrorFormatter,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let AxumJson(value) = AxumJson::<T>::from_request(req, state)
            .await
            .map_err(|e| e.into_response())?;

        match value.validate() {
            Ok(()) => Ok(ValidatedJson(value, std::marker::PhantomData)),
            Err(errors) => Err(F::format(errors)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Validate)]
    struct TestDto {
        #[validate(length(min = 3, max = 10))]
        name: String,
    }

    async fn handler(ValidatedJson(body, ..): ValidatedJson<TestDto>) -> Response {
        (AxumStatusCode::OK, AxumJson(body)).into_response()
    }

    #[tokio::test]
    async fn validated_json_accepts_valid_body() {
        let app = Router::new().route("/", post(handler));
        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"alice"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn validated_json_rejects_invalid_body_with_structured_error() {
        let app = Router::new().route("/", post(handler));
        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"ab"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxumStatusCode::BAD_REQUEST);

        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"], "Validation error");
        assert!(body["details"]["name"].is_array());
    }

    #[derive(Default)]
    struct CustomFormatter;

    impl ValidationErrorFormatter for CustomFormatter {
        fn format(errors: ValidationErrors) -> Response {
            crate::Http::bad_request_with(serde_json::json!({
                "validation_failed": true,
                "messages": errors,
            }))
        }
    }

    async fn custom_handler(
        ValidatedJson(body, ..): ValidatedJson<TestDto, CustomFormatter>,
    ) -> Response {
        (AxumStatusCode::OK, AxumJson(body)).into_response()
    }

    #[tokio::test]
    async fn validated_json_uses_custom_formatter() {
        let app = Router::new().route("/", post(custom_handler));
        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"ab"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxumStatusCode::BAD_REQUEST);

        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["validation_failed"], true);
        assert!(body["messages"]["name"].is_array());
    }
}
