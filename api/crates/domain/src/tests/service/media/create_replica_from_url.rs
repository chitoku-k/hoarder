use std::io::Cursor;

use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use tokio_util::task::TaskTracker;
use uuid::uuid;

use crate::{
    entity::{
        media::MediumId,
        objects::{Entry, EntryKind, EntryMetadata, EntryUrl},
        replicas::{OriginalImage, Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId, ThumbnailImage},
    },
    error::{Error, ErrorKind},
    service::media::{MediaService, MediaServiceInterface, MediumSource},
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
    let mock_sources_repository = MockSourcesRepository::new();
    let task_tracker = TaskTracker::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_medium_image_processor = MockMediumImageProcessor::new();
            mock_medium_image_processor
                .expect_generate_thumbnail()
                .times(1)
                .returning(|_| Ok((
                    OriginalImage::new("image/png", Size::new(720, 720)),
                    ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
                )));

            mock_medium_image_processor
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(ok((
                Entry::new(
                    "77777777-7777-7777-7777-777777777777.png".to_string(),
                    Some(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
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

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail_image, original_url, original_image, status| {
            (medium_id, thumbnail_image, original_url, original_image, status) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &None,
                "file:///77777777-7777-7777-7777-777777777777.png",
                &None,
                &ReplicaStatus::Processing,
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: None,
                size: None,
                status: ReplicaStatus::Processing,
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    mock_replicas_repository
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_replicas_repository = MockReplicasRepository::new();
            mock_replicas_repository
                .expect_update_by_id()
                .times(1)
                .withf(|id, thumbnail_image, original_url, original_image, status| {
                    (id, thumbnail_image, original_url, original_image, status) == (
                        &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                        &Some(Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)))),
                        &None,
                        &Some(Some(OriginalImage::new("image/png", Size::new(720, 720)))),
                        &Some(ReplicaStatus::Ready),
                    )
                })
                .returning(|_, _, _, _, _| {
                    Box::pin(ok(Replica {
                        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                        display_order: 1,
                        thumbnail: Some(Thumbnail {
                            id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                            size: Size::new(240, 240),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                        }),
                        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                        mime_type: Some("image/png".to_string()),
                        size: Some(Size::new(720, 720)),
                        status: ReplicaStatus::Ready,
                        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                    }))
                });

            mock_replicas_repository
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::<Cursor<&[_]>>::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Processing,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });

    task_tracker.close();
    task_tracker.wait().await;
}

#[tokio::test]
async fn succeeds_and_process_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let task_tracker = TaskTracker::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_medium_image_processor = MockMediumImageProcessor::new();
            mock_medium_image_processor
                .expect_generate_thumbnail()
                .times(1)
                .returning(|_| Err(Error::new(ErrorKind::MediumReplicaUnsupported, anyhow!("unsupported"))));

            mock_medium_image_processor
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(ok((
                Entry::new(
                    "77777777-7777-7777-7777-777777777777.png".to_string(),
                    Some(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
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

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail_image, original_url, original_image, status| {
            (medium_id, thumbnail_image, original_url, original_image, status) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &None,
                "file:///77777777-7777-7777-7777-777777777777.png",
                &None,
                &ReplicaStatus::Processing,
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: None,
                size: None,
                status: ReplicaStatus::Processing,
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    mock_replicas_repository
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_replicas_repository = MockReplicasRepository::new();
            mock_replicas_repository
                .expect_update_by_id()
                .times(1)
                .withf(|id, thumbnail_image, original_url, original_image, status| {
                    (id, thumbnail_image, original_url, original_image, status) == (
                        &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                        &Some(None),
                        &None,
                        &Some(None),
                        &Some(ReplicaStatus::Error),
                    )
                })
                .returning(|_, _, _, _, _| {
                    Box::pin(ok(Replica {
                        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                        display_order: 1,
                        thumbnail: None,
                        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                        mime_type: None,
                        size: None,
                        status: ReplicaStatus::Error,
                        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                    }))
                });

            mock_replicas_repository
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::<Cursor<&[_]>>::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Processing,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });

    task_tracker.close();
    task_tracker.wait().await;
}

#[tokio::test]
async fn succeeds_and_update_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let task_tracker = TaskTracker::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_medium_image_processor = MockMediumImageProcessor::new();
            mock_medium_image_processor
                .expect_generate_thumbnail()
                .times(1)
                .returning(|_| Err(Error::new(ErrorKind::MediumReplicaUnsupported, anyhow!("unsupported"))));

            mock_medium_image_processor
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(ok((
                Entry::new(
                    "77777777-7777-7777-7777-777777777777.png".to_string(),
                    Some(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
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

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail_image, original_url, original_image, status| {
            (medium_id, thumbnail_image, original_url, original_image, status) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &None,
                "file:///77777777-7777-7777-7777-777777777777.png",
                &None,
                &ReplicaStatus::Processing,
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: None,
                size: None,
                status: ReplicaStatus::Processing,
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    mock_replicas_repository
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_replicas_repository = MockReplicasRepository::new();
            mock_replicas_repository
                .expect_update_by_id()
                .times(1)
                .withf(|id, thumbnail_image, original_url, original_image, status| {
                    (id, thumbnail_image, original_url, original_image, status) == (
                        &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                        &Some(None),
                        &None,
                        &Some(None),
                        &Some(ReplicaStatus::Error),
                    )
                })
                .returning(|_, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

            mock_replicas_repository
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::<Cursor<&[_]>>::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Processing,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });

    task_tracker.close();
    task_tracker.wait().await;
}

#[tokio::test]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let task_tracker = TaskTracker::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(MockMediumImageProcessor::new);

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(ok((
                Entry::new(
                    "77777777-7777-7777-7777-777777777777.png".to_string(),
                    Some(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
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

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail_image, original_url, original_image, status| {
            (medium_id, thumbnail_image, original_url, original_image, status) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &None,
                "file:///77777777-7777-7777-7777-777777777777.png",
                &None,
                &ReplicaStatus::Processing,
            )
        })
        .returning(|_, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::<Cursor<&[_]>>::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
    assert!(task_tracker.is_empty());
}

#[tokio::test]
async fn fails_with_no_url() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let task_tracker = TaskTracker::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(MockMediumImageProcessor::new);

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(ok((
                Entry::new(
                    "77777777-7777-7777-7777-777777777777.png".to_string(),
                    None,
                    EntryKind::Object,
                    None,
                ),
                Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]),
            )))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::<Cursor<&[_]>>::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectPathInvalid);
    assert!(task_tracker.is_empty());
}

#[tokio::test]
async fn fails_with_no_entry() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let task_tracker = TaskTracker::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(MockMediumImageProcessor::new);

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_get()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(err(Error::new(
                ErrorKind::ObjectGetFailed { url: "file:///77777777-7777-7777-7777-777777777777.png".to_string() },
                anyhow!("No such file or directory"),
            )))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker.clone());
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::<Cursor<&[_]>>::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectGetFailed { url } if url == "file:///77777777-7777-7777-7777-777777777777.png");
    assert!(task_tracker.is_empty());
}
