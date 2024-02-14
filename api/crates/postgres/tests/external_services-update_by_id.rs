use domain::{
    entity::external_services::ExternalServiceId,
    error::ErrorKind,
    repository::external_services::ExternalServicesRepository,
};
use postgres::external_services::PostgresExternalServicesRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
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
async fn with_name_succeeds(ctx: &DatabaseContext) {
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
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some("PIXIV"),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ExternalServiceNotFound { id } if id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")));
}
