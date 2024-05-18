use domain::{
    error::ErrorKind,
    repository::external_services::ExternalServicesRepository,
};
use postgres::external_services::PostgresExternalServicesRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.create("foobar", "foobar", "FooBar", Some("https://foobar.example.com")).await.unwrap();

    assert_eq!(actual.slug, "foobar".to_string());
    assert_eq!(actual.kind, "foobar".to_string());
    assert_eq!(actual.name, "FooBar".to_string());
    assert_eq!(actual.base_url, Some("https://foobar.example.com".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url" FROM "external_services" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "foobar");
    assert_eq!(actual.get::<&str, &str>("kind"), "foobar");
    assert_eq!(actual.get::<&str, &str>("name"), "FooBar");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), Some("https://foobar.example.com"));
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.create("x", "x", "X", Some("https://x.com")).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ExternalServiceSlugDuplicate { slug } if slug == "x");
}
