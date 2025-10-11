use domain::{
    entity::external_services::ExternalServiceKind,
    error::ErrorKind,
    repository::external_services::ExternalServicesRepository,
};
use insta::assert_toml_snapshot;
use postgres::external_services::PostgresExternalServicesRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use tracing::Instrument;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.create(
        "foobar",
        ExternalServiceKind::Custom("foobar".to_string()),
        "FooBar",
        Some("https://foobar.example.com"),
        Some(r"^https?://foobar\.example\.com/(?<id>\d+)(?:[/?#].*)?$"),
    ).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual.slug, "foobar".to_string());
    assert_eq!(actual.kind, ExternalServiceKind::Custom("foobar".to_string()));
    assert_eq!(actual.name, "FooBar".to_string());
    assert_eq!(actual.base_url, Some("https://foobar.example.com".to_string()));
    assert_eq!(actual.url_pattern, Some(r"^https?://foobar\.example\.com/(?<id>\d+)(?:[/?#].*)?$".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "slug", "kind", "name", "base_url", "url_pattern" FROM "external_services" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "foobar");
    assert_eq!(actual.get::<&str, &str>("kind"), "foobar");
    assert_eq!(actual.get::<&str, &str>("name"), "FooBar");
    assert_eq!(actual.get::<Option<&str>, &str>("base_url"), Some("https://foobar.example.com"));
    assert_eq!(actual.get::<Option<&str>, &str>("url_pattern"), Some(r"^https?://foobar\.example\.com/(?<id>\d+)(?:[/?#].*)?$"));

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.create(
        "x",
        ExternalServiceKind::X,
        "X",
        Some("https://x.com"),
        Some(r"^https?://foobar\.example\.com/(?<id>\d+)(?:[/?#].*)?$"),
    ).instrument(ctx.span.clone()).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ExternalServiceSlugDuplicate { slug } if slug == "x");

    assert_toml_snapshot!(ctx.queries());
}
