use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use ordermap::OrderMap;
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::{media::{Medium, MediumId}, tags::TagDepth},
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
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone_box().eq([
                MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ]) &&
            (tag_depth, replicas, sources) == (
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
            ]))
        });

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_media_by_ids(
        [
            MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
        ].into_iter(),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone_box().eq([
                MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ]) &&
            (tag_depth, replicas, sources) == (
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
    let actual = service.get_media_by_ids(
        [
            MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
        ].into_iter(),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
