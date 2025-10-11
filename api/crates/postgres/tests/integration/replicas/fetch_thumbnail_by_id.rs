use domain::{
    entity::replicas::ThumbnailId,
    error::ErrorKind,
    repository::replicas::ReplicasRepository,
};
use insta::assert_toml_snapshot;
use postgres::replicas::PostgresReplicasRepository;
use pretty_assertions::{assert_eq, assert_matches};
use test_context::test_context;
use tracing::Instrument;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_thumbnail_by_id(
        ThumbnailId::from(uuid!("9785df5f-f975-4253-9b50-b5e3abb92a70")),
    ).instrument(ctx.span.clone()).await.unwrap();

    assert_eq!(actual, vec![
        0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x20,
        0x18, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x9d, 0x01, 0x2a, 0x01, 0x00, 0x01, 0x00, 0x02, 0x00,
        0x34, 0x25, 0xa4, 0x00, 0x03, 0x70, 0x00, 0xfe, 0xfb, 0xfd, 0x50, 0x00,
    ]);

    assert_toml_snapshot!(ctx.queries());
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_thumbnail_by_id(
        ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
    ).instrument(ctx.span.clone()).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ThumbnailNotFound { id } if id == &ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")));

    assert_toml_snapshot!(ctx.queries());
}
