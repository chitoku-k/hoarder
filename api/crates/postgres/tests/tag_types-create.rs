use domain::repository::tag_types::TagTypesRepository;
use postgres::tag_types::PostgresTagTypesRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.create("foobar", "FooBar").await.unwrap();

    assert_eq!(actual.slug, "foobar".to_string());
    assert_eq!(actual.name, "FooBar".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "tag_types" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "foobar");
    assert_eq!(actual.get::<&str, &str>("name"), "FooBar");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.create("character", "キャラクター").await;

    assert!(actual.is_err());
}
