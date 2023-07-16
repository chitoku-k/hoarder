use domain::{
    entity::tag_types::{TagType, TagTypeId},
    repository::{tag_types::TagTypesRepository, DeleteResult},
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
async fn create_succeeds(ctx: &DatabaseContext) {
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
async fn create_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.create("character", "キャラクター").await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all().await.unwrap();

    assert_eq!(actual, vec![
        TagType {
            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            slug: "character".to_string(),
            name: "キャラクター".to_string(),
        },
        TagType {
            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            slug: "work".to_string(),
            name: "作品".to_string(),
        },
        TagType {
            id: TagTypeId::from(uuid!("37553a79-53cd-4768-8a06-1378d6010954")),
            slug: "clothes".to_string(),
            name: "衣装".to_string(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_succeeds(ctx: &DatabaseContext) {
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
async fn update_by_id_with_slug_and_name_succeeds(ctx: &DatabaseContext) {
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
async fn update_by_id_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagTypeId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some("illustrators"),
        Some("絵師"),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tag_types" WHERE "id" = $1"#)
        .bind(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
