use domain::{
    entity::media::MediumId,
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
    let actual = repository.create(
        MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
        Some(vec![0x01, 0x02, 0x03, 0x04]),
        "file:///var/lib/hoarder/replica01.png",
        "image/png",
    ).await.unwrap();

    assert_eq!(actual.display_order, Some(1));
    assert_eq!(actual.has_thumbnail, true);
    assert_eq!(actual.original_url, "file:///var/lib/hoarder/replica01.png".to_string());
    assert_eq!(actual.mime_type, "image/png".to_string());

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "thumbnail", "original_url", "mime_type" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 1);
    assert_eq!(actual.get::<Option<Vec<u8>>, &str>("thumbnail"), Some(vec![0x01, 0x02, 0x03, 0x04]));
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///var/lib/hoarder/replica01.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn first_replica_without_thumbnail_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.create(
        MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
        None,
        "file:///var/lib/hoarder/replica01.png",
        "image/png",
    ).await.unwrap();

    assert_eq!(actual.display_order, Some(1));
    assert_eq!(actual.has_thumbnail, false);
    assert_eq!(actual.original_url, "file:///var/lib/hoarder/replica01.png".to_string());
    assert_eq!(actual.mime_type, "image/png".to_string());

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "thumbnail", "original_url", "mime_type" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 1);
    assert_eq!(actual.get::<Option<Vec<u8>>, &str>("thumbnail"), None);
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///var/lib/hoarder/replica01.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn non_first_replica_with_thumbnail_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.create(
        MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
        Some(vec![0x01, 0x02, 0x03, 0x04]),
        "file:///var/lib/hoarder/replica02.png",
        "image/png",
    ).await.unwrap();

    assert_eq!(actual.display_order, Some(3));
    assert_eq!(actual.has_thumbnail, true);
    assert_eq!(actual.original_url, "file:///var/lib/hoarder/replica02.png".to_string());
    assert_eq!(actual.mime_type, "image/png".to_string());

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "thumbnail", "original_url", "mime_type" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 3);
    assert_eq!(actual.get::<Option<Vec<u8>>, &str>("thumbnail"), Some(vec![0x01, 0x02, 0x03, 0x04]));
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///var/lib/hoarder/replica02.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn non_first_replica_without_thumbnail_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.create(
        MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
        None,
        "file:///var/lib/hoarder/replica02.png",
        "image/png",
    ).await.unwrap();

    assert_eq!(actual.display_order, Some(3));
    assert_eq!(actual.has_thumbnail, false);
    assert_eq!(actual.original_url, "file:///var/lib/hoarder/replica02.png".to_string());
    assert_eq!(actual.mime_type, "image/png".to_string());

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "thumbnail", "original_url", "mime_type" FROM "replicas" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 3);
    assert_eq!(actual.get::<Option<Vec<u8>>, &str>("thumbnail"), None);
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///var/lib/hoarder/replica02.png");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/png");
}
