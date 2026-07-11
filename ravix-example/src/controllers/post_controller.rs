use std::sync::Arc;

use ravix::prelude::*;
use ravix_logger::Logger;

use crate::middleware::jwt_guard;
use crate::models::post::{CreatePostDto, UpdatePostDto};
use crate::models::user::Claims;
use crate::services::PostService;

#[injectable]
pub struct PostController {
    #[inject]
    svc: Arc<PostService>,
    #[inject]
    logger: Arc<Logger>,
}

#[controller("/posts")]
impl PostController {
    /// GET /posts
    #[get("/")]
    pub async fn list_posts(&self) -> Response {
        match self.svc.list().await {
            Ok(posts) => Http::json(posts),
            Err(e) => e.into_response(),
        }
    }

    /// GET /posts/:id
    #[get("/:id")]
    pub async fn get_post(&self, Path(id): Path<String>) -> Response {
        match self.svc.get(&id).await {
            Ok(post) => Http::json(post),
            Err(e) => e.into_response(),
        }
    }

    /// POST /posts (auth required)
    #[post("/")]
    #[middleware(jwt_guard)]
    pub async fn create_post(
        &self,
        Extension(claims): Extension<Claims>,
        Json(body): Json<CreatePostDto>,
    ) -> Response {
        if let Err(e) = validator::Validate::validate(&body) {
            return Http::bad_request(&format!("Validation error: {:?}", e));
        }
        match self.svc.create(&claims.sub, body).await {
            Ok(post) => Http::created(post),
            Err(e) => e.into_response(),
        }
    }

    /// PUT /posts/:id (auth + owner)
    #[put("/:id")]
    #[middleware(jwt_guard)]
    pub async fn update_post(
        &self,
        Path(id): Path<String>,
        Extension(claims): Extension<Claims>,
        Json(body): Json<UpdatePostDto>,
    ) -> Response {
        match self.svc.update(&claims.sub, &id, body).await {
            Ok(post) => Http::json(post),
            Err(e) => e.into_response(),
        }
    }

    /// DELETE /posts/:id (auth + owner)
    #[delete("/:id")]
    #[middleware(jwt_guard)]
    pub async fn delete_post(
        &self,
        Path(id): Path<String>,
        Extension(claims): Extension<Claims>,
    ) -> Response {
        match self.svc.delete(&claims.sub, &id).await {
            Ok(()) => Http::no_content(),
            Err(e) => e.into_response(),
        }
    }
}