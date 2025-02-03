use anyhow::anyhow;
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::sources::SourceId,
    error::{Error, ErrorKind},
    repository::DeleteResult,
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
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_source_by_id(SourceId::from(uuid!("11111111-1111-1111-1111-111111111111"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_source_by_id(SourceId::from(uuid!("11111111-1111-1111-1111-111111111111"))).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
