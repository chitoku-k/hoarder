use domain::{
    entity::tag_types::TagTypeId,
    repository::tag_types::TagTypesRepository,
};
use postgres::tag_types::PostgresTagTypesRepository;
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
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
        None,
        None,
    ).await.unwrap();

    assert_eq!(actual.slug, "work".to_string());
    assert_eq!(actual.name, "作品".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "work");
    assert_eq!(actual.get::<&str, &str>("name"), "作品");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_slug_and_name_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
        Some("works"),
        Some("版権"),
    ).await.unwrap();

    assert_eq!(actual.slug, "works".to_string());
    assert_eq!(actual.name, "版権".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "works");
    assert_eq!(actual.get::<&str, &str>("name"), "版権");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagTypeId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some("illustrators"),
        Some("絵師"),
    ).await;

    assert!(actual.is_err());
}
