use async_trait::async_trait;
use bson::{doc, oid::ObjectId, Document};
use chrono::Utc;
 use futures::TryStreamExt;
use mongodb::Database;
use ravix::injectable;
use serde_json::json;

use crate::errors::AppError;
use crate::models::post::{CreatePostDto, Post, UpdatePostDto};
use ravix_apm::Apm;

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Post>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Post>, AppError>;
    async fn save(&self, author_id: &str, dto: CreatePostDto) -> Result<Post, AppError>;
    async fn update(&self, id: &str, author_id: &str, dto: UpdatePostDto) -> Result<Option<Post>, AppError>;
    async fn delete(&self, id: &str, author_id: &str) -> Result<bool, AppError>;
}

fn doc_to_post(doc: Document) -> Result<Post, AppError> {
    Ok(Post {
        id: doc.get_object_id("_id")?.to_hex(),
        author_id: doc.get_str("author_id")?.to_string(),
        title: doc.get_str("title")?.to_string(),
        body: doc.get_str("body")?.to_string(),
        created_at: doc.get_datetime("created_at")?.to_chrono(),
        updated_at: doc.get_datetime("updated_at")?.to_chrono(),
    })
}

#[injectable]
pub struct MongoPostRepository {
    #[inject]
    db: Database,
}

#[async_trait]
impl PostRepository for MongoPostRepository {
    async fn find_all(&self) -> Result<Vec<Post>, AppError> {
        Apm::wrap_span_future(
            "mongo.posts.find",
            "db",
            Some(
                [
                    ("collection".into(), json!("posts")),
                    ("op".into(), json!("find")),
                ]
                .into(),
            ),
            async {
                let coll = self.db.collection::<Document>("posts");
                let cursor = coll.find(doc! {}).await?;
                let docs: Vec<Document> = cursor.try_collect().await?;
                docs.into_iter().map(doc_to_post).collect()
            },
        )
        .await
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Post>, AppError> {
        Apm::wrap_span_future(
            "mongo.posts.find_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("posts")),
                    ("op".into(), json!("find_one")),
                    ("id".into(), json!(id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid post id".into()))?;
                let coll = self.db.collection::<Document>("posts");
                coll.find_one(doc! { "_id": oid })
                    .await?
                    .map(doc_to_post)
                    .transpose()
            },
        )
        .await
    }

    async fn save(&self, author_id: &str, dto: CreatePostDto) -> Result<Post, AppError> {
        Apm::wrap_span_future(
            "mongo.posts.insert_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("posts")),
                    ("op".into(), json!("insert_one")),
                ]
                .into(),
            ),
            async {
                let now = Utc::now();
                let coll = self.db.collection::<Document>("posts");
                let doc = doc! {
                    "author_id": author_id,
                    "title": &dto.title,
                    "body": &dto.body,
                    "created_at": bson::DateTime::from_chrono(now),
                    "updated_at": bson::DateTime::from_chrono(now),
                };
                let result = coll.insert_one(doc).await?;
                let oid = result
                    .inserted_id
                    .as_object_id()
                    .ok_or_else(|| AppError::InternalError("Failed to get inserted id".into()))?;
                Ok(Post {
                    id: oid.to_hex(),
                    author_id: author_id.to_string(),
                    title: dto.title,
                    body: dto.body,
                    created_at: now,
                    updated_at: now,
                })
            },
        )
        .await
    }

    async fn update(&self, id: &str, author_id: &str, dto: UpdatePostDto) -> Result<Option<Post>, AppError> {
        Apm::wrap_span_future(
            "mongo.posts.update_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("posts")),
                    ("op".into(), json!("update_one")),
                    ("id".into(), json!(id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid post id".into()))?;
                let coll = self.db.collection::<Document>("posts");

                // Build set fields
                let mut set_fields = Document::new();
                set_fields.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));
                if let Some(ref title) = dto.title {
                    set_fields.insert("title", title);
                }
                if let Some(ref body) = dto.body {
                    set_fields.insert("body", body);
                }
                if set_fields.len() == 1 {
                    // Nothing besides updated_at; return as-is
                    return coll
                        .find_one(doc! { "_id": oid, "author_id": author_id })
                        .await?
                        .map(doc_to_post)
                        .transpose();
                }

                let result = coll
                    .update_one(
                        doc! { "_id": oid, "author_id": author_id },
                        doc! { "$set": set_fields },
                    )
                    .await?;

                if result.matched_count == 0 {
                    return Ok(None);
                }

                coll.find_one(doc! { "_id": oid })
                    .await?
                    .map(doc_to_post)
                    .transpose()
            },
        )
        .await
    }

    async fn delete(&self, id: &str, author_id: &str) -> Result<bool, AppError> {
        Apm::wrap_span_future(
            "mongo.posts.delete_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("posts")),
                    ("op".into(), json!("delete_one")),
                    ("id".into(), json!(id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid post id".into()))?;
                let coll = self.db.collection::<Document>("posts");
                let result = coll
                    .delete_one(doc! { "_id": oid, "author_id": author_id })
                    .await?;
                Ok(result.deleted_count > 0)
            },
        )
        .await
    }
}
