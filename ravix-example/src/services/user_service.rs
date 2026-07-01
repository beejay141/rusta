use std::sync::Arc;
use uuid::Uuid;

use crate::models::{CreateUserDto, User};
use crate::repositories::UserRepository;

/// Business-logic layer. The `repo` field is resolved from the DI container via `#[inject]`.
#[ravix::injectable]
pub struct UserService {
    #[inject]
    pub repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub async fn find_all(&self) -> Vec<User> {
        self.repo.find_all().await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Option<User> {
        self.repo.find_by_id(id).await
    }

    pub async fn create(&self, dto: CreateUserDto) -> User {
        self.repo.save(dto).await
    }
}
