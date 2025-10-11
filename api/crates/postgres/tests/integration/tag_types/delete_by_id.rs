use domain::{
    entity::tag_types::TagTypeId,
    repository::{tag_types::TagTypesRepository, DeleteResult},
};
use insta::assert_toml_snapshot;
use postgres::tag_types::PostgresTagTypesRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use tracing::Instrument;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);

    assert_toml_snapshot!(ctx.queries());
}
