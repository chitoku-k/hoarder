use domain::{
    entity::media::MediumId,
    repository::{media::MediaRepository, DeleteResult},
};
use insta::assert_toml_snapshot;
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use tracing::Instrument;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "media" WHERE "id" = $1"#)
        .bind(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1"))).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);

    assert_toml_snapshot!(ctx.queries());
}
