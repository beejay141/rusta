use std::sync::Arc;

use rusta::injectable;
use serde_json::json;

use crate::errors::AppError;
use crate::models::comment::{Comment, CreateCommentDto, UpdateCommentDto};
use crate::repositories::CommentRepository;
use rusta_apm::Apm;

#[injectable]
pub struct CommentService {
    #[inject]
    repo: Arc<dyn CommentRepository>,
    #[inject]
    apm: Arc<Apm>,
}

impl CommentService {
    pub async fn list_for_post(&self, post_id: &str) -> Result<Vec<Comment>, AppError> {
        let handle = self.apm.start_span(
            "comment.list",
            "app",
            Some([("post_id".into(), json!(post_id))].into()),
        );
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
        self.apm
            .wrap_span_future(
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
        self.apm
            .wrap_span_future(
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
                    updated
                        .ok_or_else(|| AppError::Forbidden("Not the owner of this comment".into()))
                },
            )
            .await
    }

    pub async fn delete(&self, author_id: &str, id: &str) -> Result<(), AppError> {
        self.apm
            .wrap_span_future(
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
        self.apm
            .wrap_span_future(
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
        self.apm
            .wrap_span_future(
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

    pub async fn get(&self, id: &str) -> Result<Comment, AppError> {
        self.apm
            .wrap_span_future(
                "comment.get",
                "app",
                Some([("comment_id".into(), json!(id))].into()),
                async {
                    self.repo
                        .find_by_id(id)
                        .await?
                        .ok_or_else(|| AppError::NotFound("Comment not found".into()))
                },
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rusta::Injectable;
    use std::sync::Arc;

    struct DummyCommentRepo;

    #[async_trait]
    impl crate::repositories::CommentRepository for DummyCommentRepo {
        async fn find_by_post(&self, _post_id: &str) -> Result<Vec<Comment>, AppError> {
            Ok(vec![])
        }

        async fn find_by_id(&self, id: &str) -> Result<Option<Comment>, AppError> {
            Ok(Some(Comment {
                id: id.to_string(),
                post_id: "p1".to_string(),
                author_id: "a1".to_string(),
                body: "body".to_string(),
                like_count: 0,
                liked_by: vec![],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        }

        async fn save(
            &self,
            _post_id: &str,
            _author_id: &str,
            _dto: CreateCommentDto,
        ) -> Result<Comment, AppError> {
            Err(AppError::InternalError("not implemented".into()))
        }

        async fn update(
            &self,
            _id: &str,
            _author_id: &str,
            _dto: UpdateCommentDto,
        ) -> Result<Option<Comment>, AppError> {
            Ok(None)
        }

        async fn delete(&self, _id: &str, _author_id: &str) -> Result<bool, AppError> {
            Ok(true)
        }

        async fn add_like(&self, _id: &str, _user_id: &str) -> Result<Option<Comment>, AppError> {
            Ok(None)
        }

        async fn remove_like(
            &self,
            _id: &str,
            _user_id: &str,
        ) -> Result<Option<Comment>, AppError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn comment_service_get_calls_repo() {
        let mut c = rusta::Container::new();
        c.register(Arc::new(DummyCommentRepo) as Arc<dyn crate::repositories::CommentRepository>);
        c.register(rusta_apm::Apm::new());

        let svc: Arc<CommentService> = CommentService::construct(&c);

        let res = svc.get("cid").await;
        assert!(res.is_ok());
        let comment = res.unwrap();
        assert_eq!(comment.id, "cid");
    }
}
