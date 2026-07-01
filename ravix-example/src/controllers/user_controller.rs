use std::sync::Arc;

use axum::extract::{Json, Path};
use uuid::Uuid;

use ravix::prelude::*;

use crate::middleware::auth_guard;
use crate::models::CreateUserDto;
use crate::services::UserService;

#[injectable]
pub struct UserController {
    #[inject]
    pub svc: Arc<UserService>,
}

#[controller("/users")]
impl UserController {
    /// GET /users — list all users (public)
    #[get("/")]
    pub async fn list_users(&self) -> axum::response::Response {
        Response::json(self.svc.find_all().await)
    }

    /// GET /users/:id — get a single user (auth-protected)
    #[get("/:id")]
    #[middleware(auth_guard)]
    pub async fn get_user(&self, Path(id): Path<Uuid>) -> axum::response::Response {
        match self.svc.find_by_id(id).await {
            Some(user) => Response::json(user),
            None => Response::not_found("User not found"),
        }
    }

    /// POST /users — create a new user (public)
    #[post("/")]
    pub async fn create_user(&self, Json(body): Json<CreateUserDto>) -> axum::response::Response {
        Response::created(self.svc.create(body).await)
    }
}
