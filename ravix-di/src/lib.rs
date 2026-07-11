//! Dependency injection library for Ravix framework.
//!
//! This crate provides the core DI container, traits, and extractors.
//! For proc-macros (`#[injectable]`, `#[controller]`), use `ravix-di-macros`.

pub mod container;
pub mod error;
pub mod handler;

pub use container::{BindingCheck, Container, ContainerRef, Inject, Injectable};
pub use error::DiError;
pub use handler::RouteDescriptor;

// Re-export inventory so the macro-generated `::ravix_di::inventory::submit!` works.
pub use inventory;

// Re-export frequently used axum types with cleaner names.
pub use axum::{
    body::Body,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::from_fn as middleware_fn,
    middleware::Next,
    response::IntoResponse,
    response::Response,
};

/// JSON body extractor from axum.
pub use axum::extract::Json;

/// Extension extractor from axum.
pub use axum::extract::Extension;

/// Convenience prelude — glob-import this to use all HTTP method macros,
/// DI attributes, and common types without the `ravix_di::` prefix.
///
/// ```rust
/// use ravix_di::prelude::*;
/// ```
pub mod prelude {
    pub use crate::container::{Container, ContainerRef, Inject, Injectable};
    pub use crate::{Body, IntoResponse, Json, Next, Path, Query, Request, Response, StatusCode, Extension};
}