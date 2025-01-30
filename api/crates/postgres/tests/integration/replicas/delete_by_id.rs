use chrono::{DateTime, TimeZone, Utc};
use domain::{
    entity::replicas::ReplicaId,
    repository::{replicas::ReplicasRepository, DeleteResult},
};
use futures::TryStreamExt;
use postgres::replicas::PostgresReplicasRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_only_replica_succeeds(ctx: &DatabaseContext) {
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
async fn with_first_replica_succeeds(ctx: &DatabaseContext) {
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
    assert_eq!(actual[0].get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap());
    assert_ne!(actual[0].get::<DateTime<Utc>, &str>("updated_at"), Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap());

    assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
    assert_eq!(actual[1].get::<i32, &str>("display_order"), 2);
    assert_eq!(actual[1].get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap());
    assert_ne!(actual[1].get::<DateTime<Utc>, &str>("updated_at"), Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    let actual = repository.delete_by_id(ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_middle_replica_succeeds(ctx: &DatabaseContext) {
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
    assert_eq!(actual[0].get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap());
    assert_ne!(actual[0].get::<DateTime<Utc>, &str>("updated_at"), Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"));
    assert_eq!(actual[1].get::<i32, &str>("display_order"), 2);
    assert_eq!(actual[1].get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap());
    assert_ne!(actual[1].get::<DateTime<Utc>, &str>("updated_at"), Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    let actual = repository.delete_by_id(ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_last_replica_succeeds(ctx: &DatabaseContext) {
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
    assert_eq!(actual[0].get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap());
    assert_ne!(actual[0].get::<DateTime<Utc>, &str>("updated_at"), Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap());

    assert_eq!(actual[1].get::<Uuid, &str>("id"), uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b"));
    assert_eq!(actual[1].get::<i32, &str>("display_order"), 2);
    assert_eq!(actual[1].get::<DateTime<Utc>, &str>("created_at"), Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap());
    assert_ne!(actual[1].get::<DateTime<Utc>, &str>("updated_at"), Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap());

    let actual = repository.delete_by_id(ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
