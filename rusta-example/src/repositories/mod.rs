pub mod comment_repository;
pub mod post_repository;
pub mod user_repository;

pub use comment_repository::{CommentRepository, MongoCommentRepository};
pub use post_repository::{MongoPostRepository, PostRepository};
pub use user_repository::{MongoUserRepository, UserRepository};

use rusta::{Container, Injectable};

pub fn register_repositories(container: &mut Container) {
    container.register(MongoUserRepository::construct(container));
    container.register(MongoPostRepository::construct(container));
    container.register(MongoCommentRepository::construct(container));
}
