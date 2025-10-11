use domain::{
    error::ErrorKind,
    repository::tag_types::TagTypesRepository,
};
use insta::assert_toml_snapshot;
use postgres::tag_types::PostgresTagTypesRepository;
use pretty_assertions::{assert_eq, assert_matches};
use sqlx::Row;
use test_context::test_context;
use tracing::Instrument;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.create("foobar", "FooBar", "foobar").instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual.slug, "foobar".to_string());
    assert_eq!(actual.name, "FooBar".to_string());
    assert_eq!(actual.kana, "foobar".to_string());

    let actual = sqlx::query(r#"SELECT "id", "slug", "name", "kana" FROM "tag_types" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("slug"), "foobar");
    assert_eq!(actual.get::<&str, &str>("name"), "FooBar");
    assert_eq!(actual.get::<&str, &str>("kana"), "foobar");

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.create("character", "キャラクター", "キャラクター").instrument(ctx.span.clone()).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::TagTypeSlugDuplicate { slug } if slug == "character");

    assert_toml_snapshot!(ctx.queries());
}
