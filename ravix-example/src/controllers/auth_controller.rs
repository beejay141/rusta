use std::sync::Arc;

use ravix::prelude::*;
use ravix_logger::Logger;

use crate::models::user::{CreateUserDto, LoginDto};
use crate::services::AuthService;

#[injectable]
pub struct AuthController {
    #[inject]
    svc: Arc<AuthService>,
    #[inject]
    logger: Arc<Logger>,
}

#[controller("/auth")]
impl AuthController {
    /// POST /auth/register
    #[post("/register")]
    pub async fn register(&self, Json(body): Json<CreateUserDto>) -> Response {
        if let Err(e) = validator::Validate::validate(&body) {
            return Http::bad_request(&format!("Validation error: {:?}", e));
        }
        match self.svc.register(body).await {
            Ok(auth) => Http::created(auth),
            Err(e) => e.into_response(),
        }
    }

    /// POST /auth/login
    #[post("/login")]
    pub async fn login(&self, Json(body): Json<LoginDto>) -> Response {
        if let Err(e) = validator::Validate::validate(&body) {
            return Http::bad_request(&format!("Validation error: {:?}", e));
        }
        match self.svc.login(body).await {
            Ok(auth) => Http::json(auth),
            Err(e) => e.into_response(),
        }
    }
}