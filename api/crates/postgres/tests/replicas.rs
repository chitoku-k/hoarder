use chrono::{NaiveDate, NaiveDateTime};
use domain::{
    entity::{
        media::MediumId,
        replicas::{Replica, ReplicaId, ReplicaThumbnail},
    },
    repository::{replicas::ReplicasRepository, DeleteResult},
};
use futures::TryStreamExt;
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
async fn create_first_replica_with_thumbnail_succeeds(ctx: &DatabaseContext) {
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
async fn create_first_replica_without_thumbnail_succeeds(ctx: &DatabaseContext) {
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
async fn create_non_first_replica_with_thumbnail_succeeds(ctx: &DatabaseContext) {
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
async fn create_non_first_replica_without_thumbnail_succeeds(ctx: &DatabaseContext) {
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

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_ids_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids([
        ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
    ]).await.unwrap();

    assert_eq!(actual, vec![
        Replica {
            id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            display_order: Some(1),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            display_order: Some(2),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            display_order: Some(3),
            has_thumbnail: false,
            original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_original_url_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_original_url("file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png").await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        display_order: Some(1),
        has_thumbnail: true,
        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
        mime_type: "image/png".to_string(),
        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
    });
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_original_url_fails(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_original_url("file:///var/lib/hoarder/not-found.png").await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_thumbnail_by_id_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_thumbnail_by_id(
        ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
    ).await.unwrap();

    assert_eq!(actual, ReplicaThumbnail {
        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        display_order: Some(1),
        thumbnail: Some(vec![
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x20,
            0x18, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x9d, 0x01, 0x2a, 0x01, 0x00, 0x01, 0x00, 0x02, 0x00,
            0x34, 0x25, 0xa4, 0x00, 0x03, 0x70, 0x00, 0xfe, 0xfb, 0xfd, 0x50, 0x00,
        ]),
        original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
        mime_type: "image/png".to_string(),
        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
    });
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_thumbnail_by_id_fails(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_thumbnail_by_id(
        ReplicaId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        Some(vec![0x01, 0x02, 0x03, 0x04]),
        Some("file:///var/lib/hoarder/replica_new.jpg"),
        Some("image/jpeg"),
    ).await.unwrap();

    assert_eq!(actual.id, ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")));
    assert_eq!(actual.display_order, Some(1));
    assert_eq!(actual.has_thumbnail, true);
    assert_eq!(actual.original_url, "file:///var/lib/hoarder/replica_new.jpg".to_string());
    assert_eq!(actual.mime_type, "image/jpeg".to_string());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "medium_id", "display_order", "thumbnail", "original_url", "mime_type" FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("medium_id"), uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"));
    assert_eq!(actual.get::<i32, &str>("display_order"), 1);
    assert_eq!(actual.get::<Option<Vec<u8>>, &str>("thumbnail"), Some(vec![0x01, 0x02, 0x03, 0x04]));
    assert_eq!(actual.get::<&str, &str>("original_url"), "file:///var/lib/hoarder/replica_new.jpg");
    assert_eq!(actual.get::<&str, &str>("mime_type"), "image/jpeg");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_fails(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ReplicaId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        None,
        None,
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_with_only_replica_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(ReplicaId::from(uuid!("9b73469d-55fe-4017-aee8-dd8f8d7d067a"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_with_first_replica_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order", "created_at", "updated_at" FROM "replicas" WHERE "medium_id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 2);

    assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
    assert_eq!(actual[0].get::<i32, &str>("display_order"), 1);
    assert_eq!(actual[0].get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap());
    assert_ne!(actual[0].get::<NaiveDateTime, &str>("updated_at"), NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap());

    assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
    assert_eq!(actual[1].get::<i32, &str>("display_order"), 2);
    assert_eq!(actual[1].get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap());
    assert_ne!(actual[1].get::<NaiveDateTime, &str>("updated_at"), NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    let actual = repository.delete_by_id(ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_with_middle_replica_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order", "created_at", "updated_at" FROM "replicas" WHERE "medium_id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 2);

    assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
    assert_eq!(actual[0].get::<i32, &str>("display_order"), 1);
    assert_eq!(actual[0].get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap());
    assert_ne!(actual[0].get::<NaiveDateTime, &str>("updated_at"), NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
    assert_eq!(actual[1].get::<i32, &str>("display_order"), 2);
    assert_eq!(actual[1].get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap());
    assert_ne!(actual[1].get::<NaiveDateTime, &str>("updated_at"), NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    let actual = repository.delete_by_id(ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_with_last_replica_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "replicas" WHERE "id" = $1"#)
        .bind(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual: Vec<_> = sqlx::query(r#"SELECT "id", "display_order", "created_at", "updated_at" FROM "replicas" WHERE "medium_id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 2);

    assert_eq!(actual[0].get::<Uuid, &str>("id"), uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"));
    assert_eq!(actual[0].get::<i32, &str>("display_order"), 1);
    assert_eq!(actual[0].get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap());
    assert_ne!(actual[0].get::<NaiveDateTime, &str>("updated_at"), NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap());

    assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
    assert_eq!(actual[1].get::<i32, &str>("display_order"), 2);
    assert_eq!(actual[1].get::<NaiveDateTime, &str>("created_at"), NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap());
    assert_ne!(actual[1].get::<NaiveDateTime, &str>("updated_at"), NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap());

    let actual = repository.delete_by_id(ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
