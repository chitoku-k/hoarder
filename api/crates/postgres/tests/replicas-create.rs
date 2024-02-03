use domain::{
    entity::{media::MediumId, replicas::{OriginalImage, Size, ThumbnailImage}},
    repository::replicas::ReplicasRepository,
};
use postgres::replicas::PostgresReplicasRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn first_replica_with_thumbnail_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual_replica = repository.create(
        MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
        Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(1, 1))),
        "file:///replica01.png",
        OriginalImage::new("image/png", Size::new(720, 720)),
    ).await.unwrap();
    let actual_thumbnail = actual_replica.thumbnail.unwrap();

    assert_eq!(actual_replica.display_order, 1);
    assert_eq!(actual_replica.original_url, "file:///replica01.png".to_string());
    assert_eq!(actual_replica.mime_type, "image/png".to_string());
    assert_eq!(actual_replica.size.width, 720);
    assert_eq!(actual_replica.size.height, 720);

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "original_url", "mime_type", "width", "height" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual_replica.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 1);
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///replica01.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
    assert_eq!(actual.get::<i32, &str>("width"), 720);
    assert_eq!(actual.get::<i32, &str>("height"), 720);

    let actual = sqlx::query(r#"SELECT "id", "replica_id", "data", "width", "height" FROM "thumbnails" WHERE "id" = $1"#)
        .bind(*actual_thumbnail.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("replica_id"), *actual_replica.id);
    assert_eq!(actual.get::<Vec<u8>, &str>("data"), vec![0x01, 0x02, 0x03, 0x04]);
    assert_eq!(actual.get::<i32, &str>("width"), 1);
    assert_eq!(actual.get::<i32, &str>("height"), 1);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn first_replica_without_thumbnail_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual_replica = repository.create(
        MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
        None,
        "file:///replica01.png",
        OriginalImage::new("image/png", Size::new(720, 720)),
    ).await.unwrap();

    assert_eq!(actual_replica.display_order, 1);
    assert_eq!(actual_replica.thumbnail, None);
    assert_eq!(actual_replica.original_url, "file:///replica01.png".to_string());
    assert_eq!(actual_replica.mime_type, "image/png".to_string());
    assert_eq!(actual_replica.size.width, 720);
    assert_eq!(actual_replica.size.height, 720);

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "original_url", "mime_type", "width", "height" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual_replica.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 1);
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///replica01.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
    assert_eq!(actual.get::<i32, &str>("width"), 720);
    assert_eq!(actual.get::<i32, &str>("height"), 720);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn non_first_replica_with_thumbnail_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual_replica = repository.create(
        MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
        Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(1, 1))),
        "file:///replica02.png",
        OriginalImage::new("image/png", Size::new(720, 720)),
    ).await.unwrap();
    let actual_thumbnail = actual_replica.thumbnail.unwrap();

    assert_eq!(actual_replica.display_order, 3);
    assert_eq!(actual_replica.original_url, "file:///replica02.png".to_string());
    assert_eq!(actual_replica.mime_type, "image/png".to_string());

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "original_url", "mime_type", "width", "height" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual_replica.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 3);
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///replica02.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
    assert_eq!(actual.get::<i32, &str>("width"), 720);
    assert_eq!(actual.get::<i32, &str>("height"), 720);

    let actual = sqlx::query(r#"SELECT "id", "replica_id", "data", "width", "height" FROM "thumbnails" WHERE "id" = $1"#)
        .bind(*actual_thumbnail.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("replica_id"), *actual_replica.id);
    assert_eq!(actual.get::<Vec<u8>, &str>("data"), vec![0x01, 0x02, 0x03, 0x04]);
    assert_eq!(actual.get::<i32, &str>("width"), 1);
    assert_eq!(actual.get::<i32, &str>("height"), 1);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn non_first_replica_without_thumbnail_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.create(
        MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
        None,
        "file:///replica02.png",
        OriginalImage::new("image/png", Size::new(720, 720)),
    ).await.unwrap();

    assert_eq!(actual.display_order, 3);
    assert_eq!(actual.thumbnail, None);
    assert_eq!(actual.original_url, "file:///replica02.png".to_string());
    assert_eq!(actual.mime_type, "image/png".to_string());
    assert_eq!(actual.size.width, 720);
    assert_eq!(actual.size.height, 720);

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "original_url", "mime_type", "width", "height" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 3);
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///replica02.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
    assert_eq!(actual.get::<i32, &str>("width"), 720);
    assert_eq!(actual.get::<i32, &str>("height"), 720);
}
