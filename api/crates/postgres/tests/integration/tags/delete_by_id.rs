use domain::{
    entity::tags::TagId,
    error::ErrorKind,
    repository::{tags::TagsRepository, DeleteResult},
};
use insta::assert_toml_snapshot;
use postgres::tags::PostgresTagsRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use tracing::Instrument;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn root_with_recursive_fails(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository
        .delete_by_id(TagId::from(uuid!("00000000-0000-0000-0000-000000000000")), true)
        .instrument(ctx.span.clone())
        .await
        .unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagDeletingRoot);

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn root_without_recursive_fails(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository
        .delete_by_id(TagId::from(uuid!("00000000-0000-0000-0000-000000000000")), false)
        .instrument(ctx.span.clone())
        .await
        .unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagDeletingRoot);

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn node_with_recursive_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5, $6, $7)"#)
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("d1a302b5-7b49-44be-9019-ac337077786a"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 7);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")), true).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(7));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5, $6, $7)"#)
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("d1a302b5-7b49-44be-9019-ac337077786a"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")), true).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn node_without_recursive_fails(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5, $6, $7)"#)
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("d1a302b5-7b49-44be-9019-ac337077786a"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 7);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository
        .delete_by_id(TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")), false)
        .instrument(ctx.span.clone())
        .await
        .unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagChildrenExist { id, children } if (id, children) == (
        &TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
        &vec![
            TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
            TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        ]),
    );

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5, $6, $7)"#)
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("d1a302b5-7b49-44be-9019-ac337077786a"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 7);

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn leaf_with_recursive_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), true).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), true).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn leaf_without_recursive_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), false).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), false).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);

    assert_toml_snapshot!(ctx.queries());
}
