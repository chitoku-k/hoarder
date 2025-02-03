use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use ordermap::OrderMap;
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::{
        media::{Medium, MediumId},
        objects::EntryUrl,
        replicas::{Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId},
    },
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
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")), false).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn succeeds_with_delete_objects() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone_box().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &true,
                &false,
            )
        })
        .returning(|_, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: vec![
                        Replica {
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
                        },
                        Replica {
                            id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                            display_order: 2,
                            thumbnail: Some(Thumbnail {
                                id: ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                                size: Size::new(240, 240),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 4, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 5, 0).unwrap(),
                            }),
                            original_url: "file:///99999999-9999-9999-9999-999999999999.png".to_string(),
                            mime_type: Some("image/png".to_string()),
                            size: Some(Size::new(720, 720)),
                            status: ReplicaStatus::Ready,
                            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
            ]))
        });

    mock_media_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_delete()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    mock_objects_repository
        .expect_delete()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///99999999-9999-9999-9999-999999999999.png".to_string()))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn succeeds_with_delete_objects_not_found() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone_box().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &true,
                &false,
            )
        })
        .returning(|_, _, _, _| Box::pin(ok(Vec::new())));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[tokio::test]
async fn fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")), false).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn fails_with_fetching_medium() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone_box().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &true,
                &false,
            )
        })
        .returning(|_, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")), true).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn fails_with_deleting_object() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone_box().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &true,
                &false,
            )
        })
        .returning(|_, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: vec![
                        Replica {
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
                        },
                        Replica {
                            id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                            display_order: 2,
                            thumbnail: Some(Thumbnail {
                                id: ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                                size: Size::new(240, 240),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 4, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 5, 0).unwrap(),
                            }),
                            original_url: "file:///99999999-9999-9999-9999-999999999999.png".to_string(),
                            mime_type: Some("image/png".to_string()),
                            size: Some(Size::new(720, 720)),
                            status: ReplicaStatus::Ready,
                            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
            ]))
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_delete()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(err(Error::new(
                ErrorKind::ObjectDeleteFailed { url: "file:///77777777-7777-7777-7777-777777777777.png".to_string() },
                anyhow!("No such file or directory"),
            )))
        });

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")), true).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectDeleteFailed { url } if url == "file:///77777777-7777-7777-7777-777777777777.png");
}

#[tokio::test]
async fn fails_with_deleting_replica() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone_box().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &true,
                &false,
            )
        })
        .returning(|_, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: vec![
                        Replica {
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
                        },
                        Replica {
                            id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                            display_order: 2,
                            thumbnail: Some(Thumbnail {
                                id: ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                                size: Size::new(240, 240),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 4, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 5, 0).unwrap(),
                            }),
                            original_url: "file:///99999999-9999-9999-9999-999999999999.png".to_string(),
                            mime_type: Some("image/png".to_string()),
                            size: Some(Size::new(720, 720)),
                            status: ReplicaStatus::Ready,
                            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
            ]))
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_delete()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")), true).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
