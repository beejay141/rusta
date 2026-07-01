pub mod app;
pub mod container;
pub mod error;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;

pub use app::App;
pub use container::{Container, ContainerRef, Inject, Injectable};
pub use error::FrameworkError;
pub use handler::RouteDescriptor;
pub use response::Response;

// Re-export proc-macros so users import everything from `ravix`.
pub use ravix_macros::controller;
/// Mark a handler method as an HTTP DELETE route.
/// Must be used inside a [`controller`] block.
pub use ravix_macros::delete;
/// Mark a handler method as an HTTP GET route.
/// Must be used inside a [`controller`] block.
pub use ravix_macros::get;
pub use ravix_macros::injectable;
/// Attach a tower middleware function to a specific handler.
/// Must be used inside a [`controller`] block, directly above a route attribute.
pub use ravix_macros::middleware;
/// Mark a handler method as an HTTP PATCH route.
/// Must be used inside a [`controller`] block.
pub use ravix_macros::patch;
/// Mark a handler method as an HTTP POST route.
/// Must be used inside a [`controller`] block.
pub use ravix_macros::post;
/// Mark a handler method as an HTTP PUT route.
/// Must be used inside a [`controller`] block.
pub use ravix_macros::put;

// Re-export inventory so the macro-generated `::ravix::inventory::submit!` works.
pub use inventory;

// Re-export frequently used axum types.
pub use axum::{
    extract::{Json as JsonBody, Path, Query, State},
    http::StatusCode,
    middleware::from_fn as middleware_fn,
    response::IntoResponse,
    response::Response as AxumResponse,
};

/// Convenience prelude — glob-import this to use all HTTP method macros,
/// DI attributes, and common types without the `ravix::` prefix.
///
/// ```rust
/// use ravix::prelude::*;
/// ```
pub mod prelude {
    pub use crate::response::Response;
    pub use crate::{controller, delete, get, injectable, middleware, patch, post, put};
    pub use crate::{Container, ContainerRef, Inject, Injectable};
}
