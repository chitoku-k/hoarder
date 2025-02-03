use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::{future::{err, ok}, stream, StreamExt, TryFutureExt, TryStreamExt};
use ordermap::OrderMap;
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::{
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, ReplicaStatus},
        tags::TagDepth,
    },
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
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_watch_by_id()
        .times(1)
        .withf(|id, tag_depth, replicas, sources| {
            (id, tag_depth, replicas, sources) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _| {
            Box::pin(ok(stream::iter([
                Ok(Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                }),
                Ok(Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: vec![
                        Replica {
                            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                            display_order: 1,
                            thumbnail: None,
                            original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                            mime_type: None,
                            size: None,
                            status: ReplicaStatus::Processing,
                            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                }),
            ]).boxed()))
        });

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let stream = service.watch_medium_by_id(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap();

    let actual: Vec<_> = stream.try_collect().await.unwrap();
    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: vec![
                Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: None,
                    size: None,
                    status: ReplicaStatus::Processing,
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_watch_by_id()
        .times(1)
        .withf(|id, tag_depth, replicas, sources| {
            (id, tag_depth, replicas, sources) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.watch_medium_by_id(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).and_then(|stream| stream.try_collect::<Vec<_>>()).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
