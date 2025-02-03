use anyhow::anyhow;
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::replicas::ThumbnailId,
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

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_thumbnail_by_id()
        .times(1)
        .withf(|id| id == &ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")))
        .returning(|_| Box::pin(ok(vec![0x01, 0x02, 0x03, 0x04])));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_thumbnail_by_id(ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888"))).await.unwrap();

    assert_eq!(actual, vec![0x01, 0x02, 0x03, 0x04]);
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_thumbnail_by_id()
        .times(1)
        .withf(|id| id == &ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_thumbnail_by_id(ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888"))).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
