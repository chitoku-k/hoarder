use domain::{
    entity::external_services::{ExternalService, ExternalServiceId},
    repository::{external_services::ExternalServicesRepository, DeleteResult},
};
use postgres::external_services::PostgresExternalServicesRepository;
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
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.create("foobar", "FooBar").await.unwrap();

    assert_eq!(actual.slug, "foobar".to_string());
    assert_eq!(actual.name, "FooBar".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "external_services" WHERE "id" = $1"#)
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
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.create("twitter", "Twitter").await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_ids_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids([
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
    ]).await.unwrap();

    assert_eq!(actual, vec![
        ExternalService {
            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            slug: "pixiv".to_string(),
            name: "pixiv".to_string(),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
            slug: "twitter".to_string(),
            name: "Twitter".to_string(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all().await.unwrap();

    assert_eq!(actual, vec![
        ExternalService {
            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            slug: "pixiv".to_string(),
            name: "pixiv".to_string(),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
            slug: "skeb".to_string(),
            name: "Skeb".to_string(),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
            slug: "twitter".to_string(),
            name: "Twitter".to_string(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        None,
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")));
    assert_eq!(actual.slug, "pixiv".to_string());
    assert_eq!(actual.name, "pixiv".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "pixiv");
    assert_eq!(actual.get::<&str, &str>("name"), "pixiv");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_with_name_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        Some("PIXIV"),
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")));
    assert_eq!(actual.slug, "pixiv".to_string());
    assert_eq!(actual.name, "PIXIV".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "pixiv");
    assert_eq!(actual.get::<&str, &str>("name"), "PIXIV");
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_fails(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some("PIXIV"),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
