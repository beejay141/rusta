// Re-export axum extractors and HTTP types under the rusta namespace so
// users only need to import from `rusta`.
pub use axum::extract::Json;
pub use axum::extract::{Extension, Path, Query, State};
pub use axum::http::request::Parts;
pub use axum::http::{HeaderMap, Method, StatusCode};
