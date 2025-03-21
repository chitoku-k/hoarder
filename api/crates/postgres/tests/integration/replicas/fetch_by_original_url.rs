use chrono::{TimeZone, Utc};
use domain::{
    entity::replicas::{Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId},
    error::ErrorKind,
    repository::replicas::ReplicasRepository,
};
use postgres::replicas::PostgresReplicasRepository;
use pretty_assertions::{assert_eq, assert_matches};
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_original_url("file:///1706c7bb-4152-44b2-9bbb-1179d09a19be.png").await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        display_order: 1,
        thumbnail: Some(Thumbnail {
            id: ThumbnailId::from(uuid!("9785df5f-f975-4253-9b50-b5e3abb92a70")),
            size: Size::new(1, 1),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        }),
        original_url: "file:///1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
        mime_type: Some("image/png".to_string()),
        size: Some(Size::new(1920, 1600)),
        status: ReplicaStatus::Ready,
        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
    });
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_original_url("file:///not-found.png").await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ReplicaNotFoundByUrl { original_url } if original_url == "file:///not-found.png");
}
