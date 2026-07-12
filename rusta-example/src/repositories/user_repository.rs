use async_trait::async_trait;
use bson::{doc, oid::ObjectId, Document};
use chrono::Utc;
use mongodb::Database;
use rusta::injectable;
use serde_json::json;
use std::sync::Arc;

use crate::errors::AppError;
use crate::models::user::{CreateUserDto, User};
use rusta_apm::Apm;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn save(&self, dto: CreateUserDto, password_hash: String) -> Result<User, AppError>;
}

fn doc_to_user(doc: Document) -> Result<User, AppError> {
    Ok(User {
        id: doc.get_object_id("_id")?.to_hex(),
        username: doc.get_str("username")?.to_string(),
        email: doc.get_str("email")?.to_string(),
        password_hash: doc.get_str("password_hash")?.to_string(),
        created_at: doc.get_datetime("created_at")?.to_chrono(),
    })
}

#[injectable]
pub struct MongoUserRepository {
    #[inject]
    db: Database,
    #[inject]
    apm: Arc<Apm>,
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        self.apm
            .wrap_span_future(
                "mongo.users.find_one",
                "db",
                Some(
                    [
                        ("collection".into(), json!("users")),
                        ("op".into(), json!("find_one")),
                    ]
                    .into(),
                ),
                async {
                    let oid = ObjectId::parse_str(id)
                        .map_err(|_| AppError::NotFound("Invalid user id".into()))?;
                    let coll = self.db.collection::<Document>("users");
                    coll.find_one(doc! { "_id": oid })
                        .await?
                        .map(doc_to_user)
                        .transpose()
                },
            )
            .await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        self.apm
            .wrap_span_future(
                "mongo.users.find_one",
                "db",
                Some(
                    [
                        ("collection".into(), json!("users")),
                        ("op".into(), json!("find_one")),
                    ]
                    .into(),
                ),
                async {
                    let coll = self.db.collection::<Document>("users");
                    coll.find_one(doc! { "email": email })
                        .await?
                        .map(doc_to_user)
                        .transpose()
                },
            )
            .await
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        self.apm
            .wrap_span_future(
                "mongo.users.find_one",
                "db",
                Some(
                    [
                        ("collection".into(), json!("users")),
                        ("op".into(), json!("find_one")),
                    ]
                    .into(),
                ),
                async {
                    let coll = self.db.collection::<Document>("users");
                    coll.find_one(doc! { "username": username })
                        .await?
                        .map(doc_to_user)
                        .transpose()
                },
            )
            .await
    }

    async fn save(&self, dto: CreateUserDto, password_hash: String) -> Result<User, AppError> {
        self.apm
            .wrap_span_future(
                "mongo.users.insert_one",
                "db",
                Some(
                    [
                        ("collection".into(), json!("users")),
                        ("op".into(), json!("insert_one")),
                    ]
                    .into(),
                ),
                async {
                    let now = Utc::now();
                    let coll = self.db.collection::<Document>("users");
                    let doc = doc! {
                        "username": &dto.username,
                        "email": &dto.email,
                        "password_hash": &password_hash,
                        "created_at": bson::DateTime::from_chrono(now),
                    };
                    let result = coll.insert_one(doc).await?;
                    let oid = result.inserted_id.as_object_id().ok_or_else(|| {
                        AppError::InternalError("Failed to get inserted id".into())
                    })?;
                    Ok(User {
                        id: oid.to_hex(),
                        username: dto.username,
                        email: dto.email,
                        password_hash,
                        created_at: now,
                    })
                },
            )
            .await
    }
}
