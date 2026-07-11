use async_trait::async_trait;
use bson::{doc, oid::ObjectId, Document};
use chrono::Utc;
 use futures::TryStreamExt;
use mongodb::Database;
use ravix::injectable;
use serde_json::json;

use crate::errors::AppError;
use crate::models::comment::{Comment, CreateCommentDto, UpdateCommentDto};
use ravix_apm::Apm;

#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn find_by_post(&self, post_id: &str) -> Result<Vec<Comment>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Comment>, AppError>;
    async fn save(&self, post_id: &str, author_id: &str, dto: CreateCommentDto) -> Result<Comment, AppError>;
    async fn update(&self, id: &str, author_id: &str, dto: UpdateCommentDto) -> Result<Option<Comment>, AppError>;
    async fn delete(&self, id: &str, author_id: &str) -> Result<bool, AppError>;
    async fn add_like(&self, id: &str, user_id: &str) -> Result<Option<Comment>, AppError>;
    async fn remove_like(&self, id: &str, user_id: &str) -> Result<Option<Comment>, AppError>;
}

fn doc_to_comment(doc: Document) -> Result<Comment, AppError> {
    let liked_by: Vec<String> = doc
        .get_array("liked_by")
        .map(|arr| {
            arr.iter()
                .filter_map(|b| b.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(Comment {
        id: doc.get_object_id("_id")?.to_hex(),
        post_id: doc.get_str("post_id")?.to_string(),
        author_id: doc.get_str("author_id")?.to_string(),
        body: doc.get_str("body")?.to_string(),
        like_count: doc.get_i64("like_count").unwrap_or(0) as u64,
        liked_by,
        created_at: doc.get_datetime("created_at")?.to_chrono(),
        updated_at: doc.get_datetime("updated_at")?.to_chrono(),
    })
}

#[injectable]
pub struct MongoCommentRepository {
    #[inject]
    db: Database,
}

#[async_trait]
impl CommentRepository for MongoCommentRepository {
    async fn find_by_post(&self, post_id: &str) -> Result<Vec<Comment>, AppError> {
        Apm::wrap_span_future(
            "mongo.comments.find",
            "db",
            Some(
                [
                    ("collection".into(), json!("comments")),
                    ("op".into(), json!("find")),
                    ("post_id".into(), json!(post_id)),
                ]
                .into(),
            ),
            async {
                let coll = self.db.collection::<Document>("comments");
                let cursor = coll.find(doc! { "post_id": post_id }).await?;
                let docs: Vec<Document> = cursor.try_collect().await?;
                docs.into_iter().map(doc_to_comment).collect()
            },
        )
        .await
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Comment>, AppError> {
        Apm::wrap_span_future(
            "mongo.comments.find_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("comments")),
                    ("op".into(), json!("find_one")),
                    ("id".into(), json!(id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid comment id".into()))?;
                let coll = self.db.collection::<Document>("comments");
                coll.find_one(doc! { "_id": oid })
                    .await?
                    .map(doc_to_comment)
                    .transpose()
            },
        )
        .await
    }

    async fn save(&self, post_id: &str, author_id: &str, dto: CreateCommentDto) -> Result<Comment, AppError> {
        Apm::wrap_span_future(
            "mongo.comments.insert_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("comments")),
                    ("op".into(), json!("insert_one")),
                ]
                .into(),
            ),
            async {
                let now = Utc::now();
                let coll = self.db.collection::<Document>("comments");
                let doc = doc! {
                    "post_id": post_id,
                    "author_id": author_id,
                    "body": &dto.body,
                    "like_count": 0_i64,
                    "liked_by": [],
                    "created_at": bson::DateTime::from_chrono(now),
                    "updated_at": bson::DateTime::from_chrono(now),
                };
                let result = coll.insert_one(doc).await?;
                let oid = result
                    .inserted_id
                    .as_object_id()
                    .ok_or_else(|| AppError::InternalError("Failed to get inserted id".into()))?;
                Ok(Comment {
                    id: oid.to_hex(),
                    post_id: post_id.to_string(),
                    author_id: author_id.to_string(),
                    body: dto.body,
                    like_count: 0,
                    liked_by: vec![],
                    created_at: now,
                    updated_at: now,
                })
            },
        )
        .await
    }

    async fn update(&self, id: &str, author_id: &str, dto: UpdateCommentDto) -> Result<Option<Comment>, AppError> {
        Apm::wrap_span_future(
            "mongo.comments.update_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("comments")),
                    ("op".into(), json!("update_one")),
                    ("id".into(), json!(id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid comment id".into()))?;
                let coll = self.db.collection::<Document>("comments");

                let result = coll
                    .update_one(
                        doc! { "_id": oid, "author_id": author_id },
                        doc! { "$set": {
                            "body": &dto.body,
                            "updated_at": bson::DateTime::from_chrono(Utc::now()),
                        }},
                    )
                    .await?;

                if result.matched_count == 0 {
                    return Ok(None);
                }

                coll.find_one(doc! { "_id": oid })
                    .await?
                    .map(doc_to_comment)
                    .transpose()
            },
        )
        .await
    }

    async fn delete(&self, id: &str, author_id: &str) -> Result<bool, AppError> {
        Apm::wrap_span_future(
            "mongo.comments.delete_one",
            "db",
            Some(
                [
                    ("collection".into(), json!("comments")),
                    ("op".into(), json!("delete_one")),
                    ("id".into(), json!(id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid comment id".into()))?;
                let coll = self.db.collection::<Document>("comments");
                let result = coll
                    .delete_one(doc! { "_id": oid, "author_id": author_id })
                    .await?;
                Ok(result.deleted_count > 0)
            },
        )
        .await
    }

    async fn add_like(&self, id: &str, user_id: &str) -> Result<Option<Comment>, AppError> {
        Apm::wrap_span_future(
            "mongo.comments.add_like",
            "db",
            Some(
                [
                    ("collection".into(), json!("comments")),
                    ("op".into(), json!("update_one")),
                    ("id".into(), json!(id)),
                    ("user_id".into(), json!(user_id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid comment id".into()))?;
                let coll = self.db.collection::<Document>("comments");

                let result = coll
                    .update_one(
                        doc! { "_id": oid },
                        doc! {
                            "$addToSet": { "liked_by": user_id },
                            "$inc": { "like_count": 1 },
                        },
                    )
                    .await?;

                if result.matched_count == 0 {
                    return Ok(None);
                }

                coll.find_one(doc! { "_id": oid })
                    .await?
                    .map(doc_to_comment)
                    .transpose()
            },
        )
        .await
    }

    async fn remove_like(&self, id: &str, user_id: &str) -> Result<Option<Comment>, AppError> {
        Apm::wrap_span_future(
            "mongo.comments.remove_like",
            "db",
            Some(
                [
                    ("collection".into(), json!("comments")),
                    ("op".into(), json!("update_one")),
                    ("id".into(), json!(id)),
                    ("user_id".into(), json!(user_id)),
                ]
                .into(),
            ),
            async {
                let oid = ObjectId::parse_str(id)
                    .map_err(|_| AppError::NotFound("Invalid comment id".into()))?;
                let coll = self.db.collection::<Document>("comments");

                let result = coll
                    .update_one(
                        doc! { "_id": oid },
                        doc! {
                            "$pull": { "liked_by": user_id },
                            "$inc": { "like_count": -1 },
                        },
                    )
                    .await?;

                if result.matched_count == 0 {
                    return Ok(None);
                }

                coll.find_one(doc! { "_id": oid })
                    .await?
                    .map(doc_to_comment)
                    .transpose()
            },
        )
        .await
    }
}
