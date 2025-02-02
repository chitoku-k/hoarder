use std::io::Cursor;

use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use tokio_util::task::TaskTracker;

use crate::{
    entity::objects::{Entry, EntryKind, EntryMetadata, EntryUrl},
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
    let task_tracker = TaskTracker::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()))
        .returning(|_| {
            Box::pin(ok((
                Entry::new(
                    "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                    Some(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())),
                    EntryKind::Object,
                    Some(EntryMetadata::new(
                        4096,
                        Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap()),
                        Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap()),
                        Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap()),
                    )),
                ),
                Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]),
            )))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.get_object(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())).await.unwrap();

    assert_eq!(actual, Entry::new(
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        Some(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())),
        EntryKind::Object,
        Some(EntryMetadata::new(
            4096,
            Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap()),
            Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap()),
        )),
    ));
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();
    let task_tracker = TaskTracker::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()))
        .returning(|_| {
            Box::pin(err(Error::new(
                ErrorKind::ObjectNotFound { url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string() },
                anyhow!("No such file or directory"),
            )))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.get_object(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectNotFound { url } if url == "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg");
}
