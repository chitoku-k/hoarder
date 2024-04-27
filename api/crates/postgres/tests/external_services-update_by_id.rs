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
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        None,
        None,
        None,
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")));
    assert_eq!(actual.slug, "twitter".to_string());
    assert_eq!(actual.name, "Twitter".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "twitter");
    assert_eq!(actual.get::<&str, &str>("kind"), "twitter");
    assert_eq!(actual.get::<&str, &str>("name"), "Twitter");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), Some("https://twitter.com"));
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_slug_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        Some("x"),
        None,
        None,
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")));
    assert_eq!(actual.slug, "x".to_string());
    assert_eq!(actual.kind, "twitter".to_string());
    assert_eq!(actual.name, "Twitter".to_string());
    assert_eq!(actual.base_url, Some("https://twitter.com".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "x");
    assert_eq!(actual.get::<&str, &str>("kind"), "twitter");
    assert_eq!(actual.get::<&str, &str>("name"), "Twitter");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), Some("https://twitter.com"));
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_name_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        None,
        Some("X"),
        None,
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")));
    assert_eq!(actual.slug, "twitter".to_string());
    assert_eq!(actual.kind, "twitter".to_string());
    assert_eq!(actual.name, "X".to_string());
    assert_eq!(actual.base_url, Some("https://twitter.com".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "twitter");
    assert_eq!(actual.get::<&str, &str>("kind"), "twitter");
    assert_eq!(actual.get::<&str, &str>("name"), "X");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), Some("https://twitter.com"));
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_base_url_set_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        None,
        None,
        Some(Some("https://x.com")),
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")));
    assert_eq!(actual.slug, "twitter".to_string());
    assert_eq!(actual.kind, "twitter".to_string());
    assert_eq!(actual.name, "Twitter".to_string());
    assert_eq!(actual.base_url, Some("https://x.com".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "twitter");
    assert_eq!(actual.get::<&str, &str>("kind"), "twitter");
    assert_eq!(actual.get::<&str, &str>("name"), "Twitter");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), Some("https://x.com"));
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_base_url_remove_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        None,
        None,
        Some(None),
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")));
    assert_eq!(actual.slug, "twitter".to_string());
    assert_eq!(actual.kind, "twitter".to_string());
    assert_eq!(actual.name, "Twitter".to_string());
    assert_eq!(actual.base_url, None);

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "twitter");
    assert_eq!(actual.get::<&str, &str>("kind"), "twitter");
    assert_eq!(actual.get::<&str, &str>("name"), "Twitter");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), None);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_all_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        Some("x"),
        Some("X"),
        Some(Some("https://x.com")),
    ).await.unwrap();

    assert_eq!(actual.id, ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")));
    assert_eq!(actual.slug, "x".to_string());
    assert_eq!(actual.kind, "twitter".to_string());
    assert_eq!(actual.name, "X".to_string());
    assert_eq!(actual.base_url, Some("https://x.com".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url" FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "x");
    assert_eq!(actual.get::<&str, &str>("kind"), "twitter");
    assert_eq!(actual.get::<&str, &str>("name"), "X");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), Some("https://x.com"));
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        Some("X"),
        None,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ExternalServiceNotFound { id } if id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")));
}
