use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

// ── Domain ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub post_id: String,
    pub author_id: String,
    pub body: String,
    pub like_count: u64,
    pub liked_by: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCommentDto {
    #[validate(length(min = 1))]
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateCommentDto {
    #[validate(length(min = 1))]
    pub body: String,
}
