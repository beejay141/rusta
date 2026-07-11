use std::sync::Arc;

use ravix::prelude::*;
use ravix_logger::Logger;

use crate::middleware::jwt_guard;
use crate::models::comment::{CreateCommentDto, UpdateCommentDto};
use crate::models::user::Claims;
use crate::services::CommentService;

#[injectable]
pub struct CommentController {
    #[inject]
    svc: Arc<CommentService>,
    #[inject]
    logger: Arc<Logger>,
}

#[controller("/posts")]
impl CommentController {
    /// GET /posts/:post_id/comments
    #[get("/:post_id/comments")]
    pub async fn list_comments(&self, Path(post_id): Path<String>) -> Response {
        match self.svc.list_for_post(&post_id).await {
            Ok(comments) => Http::json(comments),
            Err(e) => e.into_response(),
        }
    }

    /// POST /posts/:post_id/comments (auth required)
    #[post("/:post_id/comments")]
    #[middleware(jwt_guard)]
    pub async fn create_comment(
        &self,
        Path(post_id): Path<String>,
        Extension(claims): Extension<Claims>,
        Json(body): Json<CreateCommentDto>,
    ) -> Response {
        if let Err(e) = validator::Validate::validate(&body) {
            return Http::bad_request(&format!("Validation error: {:?}", e));
        }
        match self.svc.create(&post_id, &claims.sub, body).await {
            Ok(comment) => Http::created(comment),
            Err(e) => e.into_response(),
        }
    }

    /// PUT /posts/:post_id/comments/:id (auth + owner)
    #[put("/:post_id/comments/:id")]
    #[middleware(jwt_guard)]
    pub async fn update_comment(
        &self,
        Path((_post_id, id)): Path<(String, String)>,
        Extension(claims): Extension<Claims>,
        Json(body): Json<UpdateCommentDto>,
    ) -> Response {
        if let Err(e) = validator::Validate::validate(&body) {
            return Http::bad_request(&format!("Validation error: {:?}", e));
        }
        match self.svc.update(&claims.sub, &id, body).await {
            Ok(comment) => Http::json(comment),
            Err(e) => e.into_response(),
        }
    }

    /// DELETE /posts/:post_id/comments/:id (auth + owner)
    #[delete("/:post_id/comments/:id")]
    #[middleware(jwt_guard)]
    pub async fn delete_comment(
        &self,
        Path((_post_id, id)): Path<(String, String)>,
        Extension(claims): Extension<Claims>,
    ) -> Response {
        match self.svc.delete(&claims.sub, &id).await {
            Ok(()) => Http::no_content(),
            Err(e) => e.into_response(),
        }
    }

    /// POST /posts/:post_id/comments/:id/like (auth required)
    #[post("/:post_id/comments/:id/like")]
    #[middleware(jwt_guard)]
    pub async fn like_comment(
        &self,
        Path((_post_id, id)): Path<(String, String)>,
        Extension(claims): Extension<Claims>,
    ) -> Response {
        match self.svc.like(&claims.sub, &id).await {
            Ok(comment) => Http::json(comment),
            Err(e) => e.into_response(),
        }
    }

    /// DELETE /posts/:post_id/comments/:id/like (auth required)
    #[delete("/:post_id/comments/:id/like")]
    #[middleware(jwt_guard)]
    pub async fn unlike_comment(
        &self,
        Path((_post_id, id)): Path<(String, String)>,
        Extension(claims): Extension<Claims>,
    ) -> Response {
        match self.svc.unlike(&claims.sub, &id).await {
            Ok(comment) => Http::json(comment),
            Err(e) => e.into_response(),
        }
    }
}