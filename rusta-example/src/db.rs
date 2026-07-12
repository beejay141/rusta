use mongodb::{bson::doc, Database, IndexModel};

/// Create a MongoDB client and return the named database handle.
pub async fn init_mongo(uri: &str, db_name: &str) -> Database {
    let client = mongodb::Client::with_uri_str(uri)
        .await
        .expect("Failed to connect to MongoDB");
    let db = client.database(db_name);

    create_indexes(&db).await;

    db
}

/// Create necessary indexes on the blog collections.
pub async fn create_indexes(db: &Database) {
    // Users: unique indexes on email and username
    db.collection::<mongodb::bson::Document>("users")
        .create_indexes(vec![
            IndexModel::builder().keys(doc! { "email": 1 }).build(),
            IndexModel::builder().keys(doc! { "username": 1 }).build(),
        ])
        .await
        .expect("Failed to create user indexes");

    // Comments: compound index on post_id
    db.collection::<mongodb::bson::Document>("comments")
        .create_indexes(vec![IndexModel::builder()
            .keys(doc! { "post_id": 1 })
            .build()])
        .await
        .expect("Failed to create comment indexes");

    // Posts: index on author_id
    db.collection::<mongodb::bson::Document>("posts")
        .create_indexes(vec![IndexModel::builder()
            .keys(doc! { "author_id": 1 })
            .build()])
        .await
        .expect("Failed to create post indexes");
}
