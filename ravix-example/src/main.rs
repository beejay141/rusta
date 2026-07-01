use std::sync::Arc;

use ravix::{App, Container, Injectable};

mod controllers;
mod middleware;
mod models;
mod repositories;
mod services;

use controllers::UserController;
use repositories::{InMemoryUserRepository, UserRepository};
use services::UserService;

#[tokio::main]
async fn main() {
    let mut container = Container::new();

    // ── DAL layer ─────────────────────────────────────────────────────────────
    // InMemoryUserRepository has no #[inject] fields; construct() uses Default for all.
    let repo = InMemoryUserRepository::construct(&container) as Arc<dyn UserRepository>;
    container.register(repo);

    // ── Service layer ─────────────────────────────────────────────────────────
    // UserService has #[inject] repo: Arc<dyn UserRepository>.
    // construct() calls container.resolve::<Arc<dyn UserRepository>>() automatically.
    let svc = UserService::construct(&container);
    container.register(svc);

    // ── Controller layer ───────────────────────────────────────────────────────
    // UserController has #[inject] svc: Arc<dyn IUserService>.
    let ctrl = UserController::construct(&container);
    container.register(ctrl);

    // ── Boot ──────────────────────────────────────────────────────────────────
    App::new().container(container).run("0.0.0.0:3001").await;
}
