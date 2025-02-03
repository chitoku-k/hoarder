use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
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

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, external_service_id, external_metadata| {
            (id, external_service_id, external_metadata) == (
                &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                &Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
                &Some(ExternalMetadata::Pixiv { id: 56736941 }),
            )
        })
        .returning(|_, _, _| {
            Box::pin(ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_source_by_id(
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
        Some(ExternalMetadata::Pixiv { id: 56736941 }),
    ).await.unwrap();

    assert_eq!(actual, Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            slug: "pixiv".to_string(),
            kind: "pixiv".to_string(),
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
        },
        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
    });
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, external_service_id, external_metadata| {
            (id, external_service_id, external_metadata) == (
                &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                &Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
                &Some(ExternalMetadata::Pixiv { id: 56736941 }),
            )
        })
        .returning(|_, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_source_by_id(
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
        Some(ExternalMetadata::Pixiv { id: 56736941 }),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
