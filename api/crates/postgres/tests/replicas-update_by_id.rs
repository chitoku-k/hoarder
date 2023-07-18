use chrono::NaiveDate;
use domain::{
    entity::replicas::ReplicaId,
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
async fn succeeds(ctx: &DatabaseContext) {
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
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ReplicaId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        None,
        None,
    ).await;

    assert!(actual.is_err());
}
