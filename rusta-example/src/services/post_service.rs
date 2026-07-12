use std::sync::Arc;

use rusta::injectable;
use serde_json::json;

use crate::errors::AppError;
use crate::models::post::{CreatePostDto, Post, UpdatePostDto};
use crate::repositories::PostRepository;
use rusta_apm::Apm;

#[injectable]
pub struct PostService {
    #[inject]
    repo: Arc<dyn PostRepository>,
    #[inject]
    apm: Arc<Apm>,
}

impl PostService {
    pub async fn list(&self) -> Result<Vec<Post>, AppError> {
        let handle = self.apm.start_span("post.list", "app", None);
        let posts = self.repo.find_all().await?;
        let count = posts.len();
        handle.end(Some([("count".into(), json!(count))].into()));
        Ok(posts)
    }

    pub async fn get(&self, id: &str) -> Result<Post, AppError> {
        self.apm
            .wrap_span_future(
                "post.get",
                "app",
                Some([("post_id".into(), json!(id))].into()),
                self.repo.find_by_id(id),
            )
            .await
            .transpose()
            .unwrap_or(Err(AppError::NotFound("Post not found".into())))
    }

    pub async fn create(&self, author_id: &str, dto: CreatePostDto) -> Result<Post, AppError> {
        self.apm
            .wrap_span_future(
                "post.create",
                "app",
                Some(
                    [
                        ("author_id".into(), json!(author_id)),
                        ("title_length".into(), json!(dto.title.len())),
                    ]
                    .into(),
                ),
                self.repo.save(author_id, dto),
            )
            .await
    }

    pub async fn update(
        &self,
        author_id: &str,
        id: &str,
        dto: UpdatePostDto,
    ) -> Result<Post, AppError> {
        self.apm
            .wrap_span_future(
                "post.update",
                "app",
                Some(
                    [
                        ("post_id".into(), json!(id)),
                        ("author_id".into(), json!(author_id)),
                    ]
                    .into(),
                ),
                async {
                    let updated = self.repo.update(id, author_id, dto).await?;
                    updated.ok_or_else(|| AppError::Forbidden("Not the owner of this post".into()))
                },
            )
            .await
    }

    pub async fn delete(&self, author_id: &str, id: &str) -> Result<(), AppError> {
        self.apm
            .wrap_span_future(
                "post.delete",
                "app",
                Some(
                    [
                        ("post_id".into(), json!(id)),
                        ("author_id".into(), json!(author_id)),
                    ]
                    .into(),
                ),
                async {
                    let deleted = self.repo.delete(id, author_id).await?;
                    if !deleted {
                        return Err(AppError::Forbidden("Not the owner of this post".into()));
                    }
                    Ok(())
                },
            )
            .await
    }
}
