use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use tokio_util::task::TaskTracker;
use uuid::uuid;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        sources::{Source, SourceId},
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
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();
    let task_tracker = TaskTracker::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ]))
        .returning(|_| {
            Box::pin(ok(vec![
                Source {
                    id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                    created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
                },
            ]))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker);
    let actual = service.get_sources_by_ids([
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
    ].into_iter()).await.unwrap();

    assert_eq!(actual, vec![
        Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                slug: "x".to_string(),
                kind: "x".to_string(),
                name: "X".to_string(),
                base_url: Some("https://x.com".to_string()),
                url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
            },
            external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
        },
        Source {
            id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                kind: "pixiv".to_string(),
                name: "pixiv".to_string(),
                base_url: Some("https://www.pixiv.net".to_string()),
                url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
            created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();
    let task_tracker = TaskTracker::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ]))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker);
    let actual = service.get_sources_by_ids([
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
    ].into_iter()).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
