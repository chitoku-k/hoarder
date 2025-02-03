use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use ordermap::OrderMap;
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::{media::{Medium, MediumId}, tags::TagDepth},
    error::{Error, ErrorKind},
    repository::{Direction, Order},
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
        .expect_fetch_all()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
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
    let actual = service.get_media(
        Some(TagDepth::new(1, 1)),
        true,
        true,
        Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
        Order::Ascending,
        Direction::Forward,
        10
    ).await.unwrap();

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
        .expect_fetch_all()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_media(
        Some(TagDepth::new(1, 1)),
        true,
        true,
        Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
        Order::Ascending,
        Direction::Forward,
        10
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
