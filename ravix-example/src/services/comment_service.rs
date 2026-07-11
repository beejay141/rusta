use std::sync::Arc;

use ravix::injectable;
use serde_json::json;

use crate::errors::AppError;
use crate::models::comment::{Comment, CreateCommentDto, UpdateCommentDto};
use crate::repositories::CommentRepository;
use ravix_apm::Apm;

#[injectable]
pub struct CommentService {
    #[inject]
    repo: Arc<dyn CommentRepository>,
}

impl CommentService {
    pub async fn list_for_post(&self, post_id: &str) -> Result<Vec<Comment>, AppError> {
        let handle = Apm::start_span("comment.list", "app", Some([("post_id".into(), json!(post_id))].into()));
        let comments = self.repo.find_by_post(post_id).await?;
        handle.end(Some([("count".into(), json!(comments.len()))].into()));
        Ok(comments)
    }

    pub async fn create(
        &self,
        post_id: &str,
        author_id: &str,
        dto: CreateCommentDto,
    ) -> Result<Comment, AppError> {
        Apm::wrap_span_future(
            "comment.create",
            "app",
            Some(
                [
                    ("post_id".into(), json!(post_id)),
                    ("author_id".into(), json!(author_id)),
                ]
                .into(),
            ),
            self.repo.save(post_id, author_id, dto),
        )
        .await
    }

    pub async fn update(
        &self,
        author_id: &str,
        id: &str,
        dto: UpdateCommentDto,
    ) -> Result<Comment, AppError> {
        Apm::wrap_span_future(
            "comment.update",
            "app",
            Some(
                [
                    ("comment_id".into(), json!(id)),
                    ("author_id".into(), json!(author_id)),
                ]
                .into(),
            ),
            async {
                let updated = self.repo.update(id, author_id, dto).await?;
                updated.ok_or_else(|| AppError::Forbidden("Not the owner of this comment".into()))
            },
        )
        .await
    }

    pub async fn delete(&self, author_id: &str, id: &str) -> Result<(), AppError> {
        Apm::wrap_span_future(
            "comment.delete",
            "app",
            Some(
                [
                    ("comment_id".into(), json!(id)),
                    ("author_id".into(), json!(author_id)),
                ]
                .into(),
            ),
            async {
                let deleted = self.repo.delete(id, author_id).await?;
                if !deleted {
                    return Err(AppError::Forbidden("Not the owner of this comment".into()));
                }
                Ok(())
            },
        )
        .await
    }

    pub async fn like(&self, user_id: &str, id: &str) -> Result<Comment, AppError> {
        Apm::wrap_span_future(
            "comment.like",
            "app",
            Some(
                [
                    ("comment_id".into(), json!(id)),
                    ("user_id".into(), json!(user_id)),
                ]
                .into(),
            ),
            async {
                let comment = self
                    .repo
                    .add_like(id, user_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("Comment not found".into()))?;
                Ok(comment)
            },
        )
        .await
    }

    pub async fn unlike(&self, user_id: &str, id: &str) -> Result<Comment, AppError> {
        Apm::wrap_span_future(
            "comment.unlike",
            "app",
            Some(
                [
                    ("comment_id".into(), json!(id)),
                    ("user_id".into(), json!(user_id)),
                ]
                .into(),
            ),
            async {
                let comment = self
                    .repo
                    .remove_like(id, user_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("Comment not found".into()))?;
                Ok(comment)
            },
        )
        .await
    }
}