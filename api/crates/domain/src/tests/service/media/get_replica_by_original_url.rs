use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use tokio_util::task::TaskTracker;
use uuid::uuid;

use crate::{
    entity::replicas::{Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId},
    error::{Error, ErrorKind},
    service::media::{MediaService, MediaServiceInterface},
};

use super::mocks::domain::{
    processor::media::MockMediumImageProcessor,
    repository::{
        media::MockMediaRepository,
        objects::MockObjectsRepository,
        replicas::MockReplicasRepository,
        sources::MockSourcesRepository,
    },
};

#[tokio::test]
async fn succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();
    let task_tracker = TaskTracker::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: Some(Thumbnail {
                    id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    size: Size::new(240, 240),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                }),
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: Some("image/png".to_string()),
                size: Some(Size::new(720, 720)),
                status: ReplicaStatus::Ready,
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker);
    let actual = service.get_replica_by_original_url("file:///77777777-7777-7777-7777-777777777777.png").await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: Some(Thumbnail {
            id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            size: Size::new(240, 240),
            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
        }),
        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: Some("image/png".to_string()),
        size: Some(Size::new(720, 720)),
        status: ReplicaStatus::Ready,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();
    let task_tracker = TaskTracker::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker);
    let actual = service.get_replica_by_original_url("file:///77777777-7777-7777-7777-777777777777.png").await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
