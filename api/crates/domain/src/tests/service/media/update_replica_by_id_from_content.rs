use std::io::{copy, Cursor};

use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use serial_test::serial;
use uuid::uuid;

use crate::{
    entity::{
        objects::{Entry, EntryKind, EntryMetadata, EntryUrl, EntryUrlPath},
        replicas::{OriginalImage, Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId, ThumbnailImage},
    },
    error::{Error, ErrorKind},
    repository::{objects::{ObjectOverwriteBehavior, ObjectStatus}, DeleteResult},
    service::media::{MediaService, MediaServiceInterface, MediumOverwriteBehavior, MediumSource},
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
    let mock_sources_repository = MockSourcesRepository::new();

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
                    OriginalImage::new("image/jpeg", Size::new(720, 720)),
                    ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
                )));

            mock_medium_image_processor
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _| {
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
                ObjectStatus::Created,
                Vec::new(),
            )))
        });

    mock_objects_repository
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_objects_repository = MockObjectsRepository::new();
            mock_objects_repository
                .expect_copy()
                .withf(|read, write| (read, write) == (
                    &Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
                    &Vec::new(),
                ))
                .returning(|read, write| Box::pin(ok(copy(read, write).unwrap())));

            mock_objects_repository
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image, status| {
            (id, thumbnail_image, original_url, original_image, status) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(None),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(None),
                &Some(ReplicaStatus::Processing),
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
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
                        &Some(Some(OriginalImage::new("image/jpeg", Size::new(720, 720)))),
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
                        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                        mime_type: Some("image/jpeg".to_string()),
                        size: Some(Size::new(720, 720)),
                        status: ReplicaStatus::Ready,
                        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                    }))
                });

            mock_replicas_repository
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let (actual, task) = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Processing,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });

    let actual = task.await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: Some(Thumbnail {
            id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            size: Size::new(240, 240),
            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
        }),
        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: Some("image/jpeg".to_string()),
        size: Some(Size::new(720, 720)),
        status: ReplicaStatus::Ready,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
#[serial]
async fn succeeds_and_copy_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(MockMediumImageProcessor::new);

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _| {
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
                ObjectStatus::Created,
                Vec::new(),
            )))
        });

    mock_objects_repository
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_objects_repository = MockObjectsRepository::new();
            mock_objects_repository
                .expect_copy()
                .withf(|read, write| (read, write) == (
                    &Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
                    &Vec::new(),
                ))
                .returning(|_, _| Box::pin(err(Error::other(anyhow!("No such file or directory")))));

            mock_objects_repository
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image, status| {
            (id, thumbnail_image, original_url, original_image, status) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(None),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(None),
                &Some(ReplicaStatus::Processing),
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
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
                        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                        mime_type: None,
                        size: None,
                        status: ReplicaStatus::Error,
                        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                    }))
                });

            mock_replicas_repository
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let (actual, task) = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Processing,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });

    let actual = task.await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Error,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
#[serial]
async fn succeeds_and_process_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

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
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _| {
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
                ObjectStatus::Created,
                Vec::new(),
            )))
        });

    mock_objects_repository
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_objects_repository = MockObjectsRepository::new();
            mock_objects_repository
                .expect_copy()
                .withf(|read, write| (read, write) == (
                    &Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
                    &Vec::new(),
                ))
                .returning(|read, write| Box::pin(ok(copy(read, write).unwrap())));

            mock_objects_repository
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image, status| {
            (id, thumbnail_image, original_url, original_image, status) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(None),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(None),
                &Some(ReplicaStatus::Processing),
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
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
                        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                        mime_type: None,
                        size: None,
                        status: ReplicaStatus::Error,
                        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                    }))
                });

            mock_replicas_repository
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let (actual, task) = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Processing,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });

    let actual = task.await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Error,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
#[serial]
async fn succeeds_and_update_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

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
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _| {
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
                ObjectStatus::Created,
                Vec::new(),
            )))
        });

    mock_objects_repository
        .expect_clone()
        .times(1)
        .returning(|| {
            let mut mock_objects_repository = MockObjectsRepository::new();
            mock_objects_repository
                .expect_copy()
                .withf(|read, write| (read, write) == (
                    &Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
                    &Vec::new(),
                ))
                .returning(|read, write| Box::pin(ok(copy(read, write).unwrap())));

            mock_objects_repository
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image, status| {
            (id, thumbnail_image, original_url, original_image, status) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(None),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(None),
                &Some(ReplicaStatus::Processing),
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
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

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let (actual, task) = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: None,
        original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: None,
        size: None,
        status: ReplicaStatus::Processing,
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });

    let actual = task.await.unwrap_err();
    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
#[serial]
async fn fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _| {
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
                ObjectStatus::Created,
                Vec::new(),
            )))
        });

    mock_objects_repository
        .expect_delete()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    mock_objects_repository
        .expect_clone()
        .times(1)
        .returning(MockObjectsRepository::new);

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image, status| {
            (id, thumbnail_image, original_url, original_image, status) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(None),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(None),
                &Some(ReplicaStatus::Processing),
            )
        })
        .returning(|_, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .returning(MockMediumImageProcessor::new);

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.map(|(replica, _task)| replica).unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
#[serial]
async fn fails_and_delete_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _| {
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
                ObjectStatus::Created,
                Vec::new(),
            )))
        });

    mock_objects_repository
        .expect_delete()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()))
        .returning(|_| Box::pin(err(Error::other(anyhow!("No such file or directory")))));

    mock_objects_repository
        .expect_clone()
        .times(1)
        .returning(MockObjectsRepository::new);

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image, status| {
            (id, thumbnail_image, original_url, original_image, status) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(None),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(None),
                &Some(ReplicaStatus::Processing),
            )
        })
        .returning(|_, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .returning(MockMediumImageProcessor::new);

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.map(|(replica, _task)| replica).unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
#[serial]
async fn fails_with_no_url() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _| {
            Box::pin(ok((
                Entry::new(
                    "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                    None,
                    EntryKind::Object,
                    None,
                ),
                ObjectStatus::Created,
                Vec::new(),
            )))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .returning(MockMediumImageProcessor::new);

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.map(|(replica, _task)| replica).unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectPathInvalid);
}

#[tokio::test]
#[serial]
async fn fails_with_replica_already_exists() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Fail,
            )
        })
        .returning(|_, _| {
            Box::pin(err(Error::new(
                ErrorKind::ObjectAlreadyExists { url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(), entry: None },
                anyhow!("File exists"),
            )))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|url| url == "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg")
        .returning(|_| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: Some(Thumbnail {
                    id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    size: Size::new(240, 240),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                }),
                original_url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                mime_type: Some("image/jpeg".to_string()),
                size: Some(Size::new(720, 720)),
                status: ReplicaStatus::Ready,
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(MockMediumImageProcessor::new);

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Fail,
        ),
    ).await.map(|(replica, _task)| replica).unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ReplicaOriginalUrlDuplicate { original_url, .. } if original_url == "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg");
}

#[tokio::test]
#[serial]
async fn fails_with_replica_already_exists_and_fetch_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Fail,
            )
        })
        .returning(|_, _| {
            Box::pin(err(Error::new(
                ErrorKind::ObjectAlreadyExists { url: "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(), entry: None },
                anyhow!("File exists"),
            )))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|url| url == "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg")
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_clone()
        .times(1)
        .returning(MockMediumImageProcessor::new);

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            MediumOverwriteBehavior::Fail,
        ),
    ).await.map(|(replica, _task)| replica).unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectAlreadyExists { url, entry } if url == "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg" && entry.is_none());
}
