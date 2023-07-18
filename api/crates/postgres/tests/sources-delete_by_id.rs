use domain::{
    entity::sources::SourceId,
    repository::{sources::SourcesRepository, DeleteResult},
};
use postgres::sources::PostgresSourcesRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "sources" WHERE "id" = $1"#)
        .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "sources" WHERE "id" = $1"#)
        .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
