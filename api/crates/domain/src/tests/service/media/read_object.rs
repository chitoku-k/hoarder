use anyhow::anyhow;
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};

use crate::{
    entity::objects::EntryUrl,
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
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_read()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()))
        .returning(|_| Box::pin(ok(&[0x01, 0x02, 0x03, 0x04][..])));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.read_object(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())).await.unwrap();

    assert_eq!(actual, &[0x01, 0x02, 0x03, 0x04]);
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_entry()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()))
        .returning(|_| {
            Box::pin(err(Error::new(
                ErrorKind::ObjectNotFound { url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string() },
                anyhow!("No such file or directory"),
            )))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_object(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectNotFound { url } if url == "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg");
}
