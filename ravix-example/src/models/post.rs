use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

// ── Domain ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePostDto {
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePostDto {
    #[validate(length(min = 1))]
    pub title: Option<String>,
    #[validate(length(min = 1))]
    pub body: Option<String>,
}