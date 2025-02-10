use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
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
        .expect_create()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            )
        })
        .returning(|_, _| {
            Box::pin(ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "x".to_string(),
                    kind: ExternalServiceKind::X,
                    name: "X".to_string(),
                    base_url: Some("https://x.com".to_string()),
                    url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                },
                external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_source(
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
    ).await.unwrap();

    assert_eq!(actual, Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            slug: "x".to_string(),
            kind: ExternalServiceKind::X,
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        },
        external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
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
        .expect_create()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            )
        })
        .returning(|_, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_source(
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
