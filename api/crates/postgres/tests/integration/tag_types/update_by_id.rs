use domain::{
    entity::tag_types::TagTypeId,
    error::ErrorKind,
    repository::tag_types::TagTypesRepository,
};
use postgres::tag_types::PostgresTagTypesRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
        None,
        None,
        None,
    ).await.unwrap();

    assert_eq!(actual.slug, "work".to_string());
    assert_eq!(actual.name, "作品".to_string());
    assert_eq!(actual.kana, "さくひん".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name", "kana" FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "work");
    assert_eq!(actual.get::<&str, &str>("name"), "作品");
    assert_eq!(actual.get::<&str, &str>("kana"), "さくひん");
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_slug_name_kana_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
        Some("works"),
        Some("版権"),
        Some("はんけん"),
    ).await.unwrap();

    assert_eq!(actual.slug, "works".to_string());
    assert_eq!(actual.name, "版権".to_string());
    assert_eq!(actual.kana, "はんけん".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name", "kana" FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "works");
    assert_eq!(actual.get::<&str, &str>("name"), "版権");
    assert_eq!(actual.get::<&str, &str>("kana"), "はんけん");
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagTypeId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some("illustrators"),
        Some("絵師"),
        Some("えし"),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagTypeNotFound { id } if id == &TagTypeId::from(uuid!("11111111-1111-1111-1111-111111111111")));
}
