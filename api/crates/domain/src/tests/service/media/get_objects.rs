use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use serial_test::serial;

use crate::{
    entity::objects::{Entry, EntryKind, EntryUrl, EntryUrlPath},
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
#[serial]
async fn succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_list()
        .times(1)
        .withf(|prefix| prefix == &EntryUrl::from("file:///path/to/dest".to_string()))
        .returning(|_| {
            Box::pin(ok(vec![
                Entry::new(
                    "container01".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/container01".to_string())),
                    EntryKind::Container,
                    None,
                ),
                Entry::new(
                    "container02".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/container02".to_string())),
                    EntryKind::Container,
                    None,
                ),
                Entry::new(
                    "object01".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/object01".to_string())),
                    EntryKind::Object,
                    None,
                ),
                Entry::new(
                    "object02".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/object02".to_string())),
                    EntryKind::Object,
                    None,
                ),
                Entry::new(
                    "unknown".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/unknown".to_string())),
                    EntryKind::Unknown,
                    None,
                ),
            ]))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_objects(EntryUrlPath::from("/path/to/dest".to_string()), None).await.unwrap();

    assert_eq!(actual, vec![
        Entry::new(
            "container01".to_string(),
            Some(EntryUrl::from("file:///path/to/dest/container01".to_string())),
            EntryKind::Container,
            None,
        ),
        Entry::new(
            "container02".to_string(),
            Some(EntryUrl::from("file:///path/to/dest/container02".to_string())),
            EntryKind::Container,
            None,
        ),
        Entry::new(
            "object01".to_string(),
            Some(EntryUrl::from("file:///path/to/dest/object01".to_string())),
            EntryKind::Object,
            None,
        ),
        Entry::new(
            "object02".to_string(),
            Some(EntryUrl::from("file:///path/to/dest/object02".to_string())),
            EntryKind::Object,
            None,
        ),
        Entry::new(
            "unknown".to_string(),
            Some(EntryUrl::from("file:///path/to/dest/unknown".to_string())),
            EntryKind::Unknown,
            None,
        ),
    ]);
}

#[tokio::test]
#[serial]
async fn succeeds_with_kind() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_list()
        .times(1)
        .withf(|prefix| prefix == &EntryUrl::from("file:///path/to/dest".to_string()))
        .returning(|_| {
            Box::pin(ok(vec![
                Entry::new(
                    "container01".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/container01".to_string())),
                    EntryKind::Container,
                    None,
                ),
                Entry::new(
                    "container02".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/container02".to_string())),
                    EntryKind::Container,
                    None,
                ),
                Entry::new(
                    "object01".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/object01".to_string())),
                    EntryKind::Object,
                    None,
                ),
                Entry::new(
                    "object02".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/object02".to_string())),
                    EntryKind::Object,
                    None,
                ),
                Entry::new(
                    "unknown".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/unknown".to_string())),
                    EntryKind::Unknown,
                    None,
                ),
            ]))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_objects(EntryUrlPath::from("/path/to/dest".to_string()), Some(EntryKind::Container)).await.unwrap();

    assert_eq!(actual, vec![
        Entry::new(
            "container01".to_string(),
            Some(EntryUrl::from("file:///path/to/dest/container01".to_string())),
            EntryKind::Container,
            None,
        ),
        Entry::new(
            "container02".to_string(),
            Some(EntryUrl::from("file:///path/to/dest/container02".to_string())),
            EntryKind::Container,
            None,
        ),
    ]);
}

#[tokio::test]
#[serial]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_list()
        .times(1)
        .withf(|prefix| prefix == &EntryUrl::from("file:///path/to/dest".to_string()))
        .returning(|_| Box::pin(err(Error::from(ErrorKind::ObjectGetFailed { url: "file:///path/to/dest".to_string() }))));

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_objects(EntryUrlPath::from("/path/to/dest".to_string()), None).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectGetFailed { url } if url == "file:///path/to/dest");
}
