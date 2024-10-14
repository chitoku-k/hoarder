use std::io::Cursor;

use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::{Medium, MediumId},
        objects::{Entry, EntryKind, EntryMetadata, EntryUrl, EntryUrlPath},
        replicas::{OriginalImage, Replica, ReplicaId, Size, Thumbnail, ThumbnailId, ThumbnailImage},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagDepth, TagId},
    },
    error::{Error, ErrorKind},
    processor::media::MockMediumImageProcessor,
    repository::{
        media::MockMediaRepository,
        objects::{MockObjectsRepository, ObjectOverwriteBehavior},
        replicas::MockReplicasRepository,
        sources::MockSourcesRepository,
        DeleteResult, Direction, Order,
    },
    service::media::{MediaService, MediaServiceInterface, MediumOverwriteBehavior, MediumSource},
};
use futures::future::{err, ok};
use ordermap::OrderMap;
use pretty_assertions::{assert_eq, assert_matches};
use serial_test::serial;
use tokio::io::BufReader;
use tokio_util::io::SyncIoBridge;
use uuid::uuid;

#[tokio::test]
async fn create_medium_succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_create()
        .times(1)
        .withf(|source_ids, created_at, tag_tag_type_ids, tag_depth, sources| {
            (source_ids, created_at, tag_tag_type_ids, tag_depth, sources) == (
                &vec![
                    SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                ],
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &vec![
                    (
                        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                    (
                        TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &None,
                &true,
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            slug: "x".to_string(),
                            kind: "x".to_string(),
                            name: "X".to_string(),
                            base_url: Some("https://x.com".to_string()),
                        },
                        external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
                    },
                    Source {
                        id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                            slug: "pixiv".to_string(),
                            kind: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                            base_url: Some("https://www.pixiv.net".to_string()),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                        created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
                    },
                ],
                tags: {
                    let mut tags = OrderMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                            kana: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: Default::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                            },
                        ],
                    );
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
                            kana: "キャラクター".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: Default::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                            },
                        ],
                    );
                    tags
                },
                replicas: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
            }))
        });

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_medium(
        vec![
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ],
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        vec![
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
            (
                TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
        None,
        true,
    ).await.unwrap();

    assert_eq!(actual, Medium {
        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        sources: vec![
            Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "x".to_string(),
                    kind: "x".to_string(),
                    name: "X".to_string(),
                    base_url: Some("https://x.com".to_string()),
                },
                external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            },
            Source {
                id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
            },
        ],
        tags: {
            let mut tags = OrderMap::new();
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                    kana: "キャラクター".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        name: "赤座あかり".to_string(),
                        kana: "あかざあかり".to_string(),
                        aliases: Default::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                    },
                ],
            );
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                    kana: "キャラクター".to_string(),
                },
                vec![
                    Tag {
                        id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                        name: "歳納京子".to_string(),
                        kana: "としのうきょうこ".to_string(),
                        aliases: Default::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                    },
                ],
            );
            tags
        },
        replicas: Vec::new(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
    });
}

#[tokio::test]
async fn create_medium_fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_create()
        .times(1)
        .withf(|source_ids, created_at, tag_tag_type_ids, tag_depth, sources| {
            (source_ids, created_at, tag_tag_type_ids, tag_depth, sources) == (
                &vec![
                    SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                ],
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &vec![
                    (
                        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                    (
                        TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &None,
                &true,
            )
        })
        .returning(|_, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_medium(
        vec![
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ],
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        vec![
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
            (
                TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
        None,
        true,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn create_replica_from_url_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_generate_thumbnail::<SyncIoBridge<BufReader<Cursor<&[_]>>>>()
        .times(1)
        .returning(|_| {
            Box::pin(ok((
                OriginalImage::new("image/png", Size::new(720, 720)),
                ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
            )))
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
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap(),
                    )),
                ),
                Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]),
            )))
        });

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail_image, original_url, original_image| {
            (medium_id, thumbnail_image, original_url, original_image) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240))),
                "file:///77777777-7777-7777-7777-777777777777.png",
                &OriginalImage::new("image/png", Size::new(720, 720)),
            )
        })
        .returning(|_, _, _, _| {
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
                mime_type: "image/png".to_string(),
                size: Size::new(720, 720),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: Some(Thumbnail {
            id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            size: Size::new(240, 240),
            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
        }),
        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: "image/png".to_string(),
        size: Size::new(720, 720),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
#[serial]
async fn create_replica_from_content_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_generate_thumbnail::<Cursor<Vec<_>>>()
        .times(1)
        .withf(|read| read == &Cursor::new(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]))
        .returning(|_| {
            Box::pin(ok((
                OriginalImage::new("image/png", Size::new(720, 720)),
                ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
            )))
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, _read, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _, _| {
            Box::pin(ok(Entry::new(
                "77777777-7777-7777-7777-777777777777.png".to_string(),
                Some(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
                EntryKind::Object,
                Some(EntryMetadata::new(
                    4096,
                    Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap(),
                    Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap(),
                )),
            )))
        });

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail_image, original_url, original_image| {
            (medium_id, thumbnail_image, original_url, original_image) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240))),
                "file:///77777777-7777-7777-7777-777777777777.png",
                &OriginalImage::new("image/png", Size::new(720, 720)),
            )
        })
        .returning(|_, _, _, _| {
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
                mime_type: "image/png".to_string(),
                size: Size::new(720, 720),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::Content(
            EntryUrlPath::from("/77777777-7777-7777-7777-777777777777.png".to_string()),
            vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: Some(Thumbnail {
            id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            size: Size::new(240, 240),
            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
        }),
        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: "image/png".to_string(),
        size: Size::new(720, 720),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn create_replica_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_generate_thumbnail::<SyncIoBridge<BufReader<Cursor<&[_]>>>>()
        .times(1)
        .returning(|_| {
            Box::pin(ok((
                OriginalImage::new("image/png", Size::new(720, 720)),
                ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
            )))
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
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap(),
                    )),
                ),
                Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]),
            )))
        });

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail_image, original_url, original_image| {
            (medium_id, thumbnail_image, original_url, original_image) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240))),
                "file:///77777777-7777-7777-7777-777777777777.png",
                &OriginalImage::new("image/png", Size::new(720, 720)),
            )
        })
        .returning(|_, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        MediumSource::Url(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn create_source_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_create()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            )
        })
        .returning(|_, _| {
            Box::pin(ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "x".to_string(),
                    kind: "x".to_string(),
                    name: "X".to_string(),
                    base_url: Some("https://x.com".to_string()),
                },
                external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_source(
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
    ).await.unwrap();

    assert_eq!(actual, Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
        },
        external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
    });
}

#[tokio::test]
async fn create_source_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_create()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            )
        })
        .returning(|_, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.create_source(
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn get_media_succeeds() {
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
async fn get_media_fails() {
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

#[tokio::test]
async fn get_media_by_ids_succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            (ids, tag_depth, replicas, sources) == (
                &vec![
                    MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                ],
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
        vec![
            MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
        ],
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
async fn get_media_by_ids_fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            (ids, tag_depth, replicas, sources) == (
                &vec![
                    MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                ],
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
        vec![
            MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
        ],
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn get_media_by_source_ids_succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_source_ids()
        .times(1)
        .withf(|source_ids, tag_depth, replicas, sources, since, until, order, limit| {
            (source_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                &vec![
                    SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                ],
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| {
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
            ]))
        });

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_media_by_source_ids(
        vec![
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ],
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
    ]);
}

#[tokio::test]
async fn get_media_by_source_ids_fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_source_ids()
        .times(1)
        .withf(|source_ids, tag_depth, replicas, sources, since, until, order, limit| {
            (source_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                &vec![
                    SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                ],
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_media_by_source_ids(
        vec![
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ],
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

#[tokio::test]
async fn get_media_by_tag_ids_succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_tag_ids()
        .times(1)
        .withf(|tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit| {
            (tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                &vec![
                    (
                        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                    (
                        TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| {
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
    let actual = service.get_media_by_tag_ids(
        vec![
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
            (
                TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
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
async fn get_media_by_tag_ids_fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_tag_ids()
        .times(1)
        .withf(|tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit| {
            (tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                &vec![
                    (
                        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                    (
                        TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_media_by_tag_ids(
        vec![
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
            (
                TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
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

#[tokio::test]
async fn get_replicas_by_ids_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids: &Vec<_>| {
            ids == &vec![
                ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            ]
        })
        .returning(|_| {
            Box::pin(ok(vec![
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
                    mime_type: "image/png".to_string(),
                    size: Size::new(720, 720),
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
                    mime_type: "image/png".to_string(),
                    size: Size::new(720, 720),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                },
            ]))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_replicas_by_ids(vec![
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
    ]).await.unwrap();

    assert_eq!(actual, vec![
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
            mime_type: "image/png".to_string(),
            size: Size::new(720, 720),
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
            mime_type: "image/png".to_string(),
            size: Size::new(720, 720),
            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn get_replicas_by_ids_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids: &Vec<_>| {
            ids == &vec![
                ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            ]
        })
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_replicas_by_ids(vec![
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
    ]).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn get_replica_by_original_url_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
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
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                size: Size::new(720, 720),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_replica_by_original_url("file:///77777777-7777-7777-7777-777777777777.png").await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: 1,
        thumbnail: Some(Thumbnail {
            id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            size: Size::new(240, 240),
            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
        }),
        original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: "image/png".to_string(),
        size: Size::new(720, 720),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn get_replica_by_original_url_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_replica_by_original_url("file:///77777777-7777-7777-7777-777777777777.png").await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn get_source_by_external_metadata_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_fetch_by_external_metadata()
        .times(1)
        .withf(|external_service_id, external_metadata| {
             (external_service_id, external_metadata) == (
                 &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                 &ExternalMetadata::Pixiv { id: 56736941 },
             )
        })
        .returning(|_, _| {
            Box::pin(ok(Some(Source {
                id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
            })))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_source_by_external_metadata(
         ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
         ExternalMetadata::Pixiv { id: 56736941 },
    ).await.unwrap();

    assert_eq!(actual, Some(Source {
        id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            slug: "pixiv".to_string(),
            kind: "pixiv".to_string(),
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
        },
        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
        created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
    }));
}

#[tokio::test]
async fn get_sources_by_external_metadata_not_found() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_fetch_by_external_metadata()
        .times(1)
        .withf(|external_service_id, external_metadata| {
             (external_service_id, external_metadata) == (
                 &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                 &ExternalMetadata::Pixiv { id: 56736941 },
             )
        })
        .returning(|_, _| Box::pin(ok(None)));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_source_by_external_metadata(
         ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
         ExternalMetadata::Pixiv { id: 56736941 },
    ).await.unwrap();

    assert!(actual.is_none());
}

#[tokio::test]
async fn get_sources_by_external_metadata_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_fetch_by_external_metadata()
        .times(1)
        .withf(|external_service_id, external_metadata| {
             (external_service_id, external_metadata) == (
                 &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                 &ExternalMetadata::Pixiv { id: 56736941 },
             )
        })
        .returning(|_, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_source_by_external_metadata(
         ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
         ExternalMetadata::Pixiv { id: 56736941 },
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn get_thumbnail_by_id_succeeds() {
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
async fn get_thumbnail_by_id_fails() {
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

#[tokio::test]
#[serial]
async fn get_objects_all_kinds_succeeds() {
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
async fn get_objects_with_kind_succeeds() {
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
async fn get_objects_fails() {
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
            Box::pin(err(Error::new(
                ErrorKind::ObjectGetFailed { url: "file:///path/to/dest".to_string() },
                anyhow!("failed to read dir: /path/to/dest"),
            )))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.get_objects(EntryUrlPath::from("/path/to/dest".to_string()), None).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ObjectGetFailed { url } if url == "file:///path/to/dest");
}

#[tokio::test]
async fn update_medium_by_id_succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_update_by_id()
        .times(1)
        .withf(|
            id,
            add_source_ids,
            remove_source_ids,
            add_tag_tag_type_ids,
            remove_tag_tag_type_ids,
            replica_orders,
            created_at,
            tag_depth,
            replicas,
            sources,
        | {
            (
                id,
                add_source_ids,
                remove_source_ids,
                add_tag_tag_type_ids,
                remove_tag_tag_type_ids,
                replica_orders,
                created_at,
                tag_depth,
                replicas,
                sources,
            ) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &[
                    SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                ],
                &[
                    SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                ],
                &[
                    (
                        TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &[
                    (
                        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &[
                    ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                ],
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _, _, _, _, _, _, _| {
            Box::pin(ok(Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: Vec::new(),
                tags: OrderMap::new(),
                replicas: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
            }))
        });

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_medium_by_id(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        [
            SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ],
        [
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ],
        [
            (
                TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
        [
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
        [
            ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ],
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap();

    assert_eq!(actual, Medium {
        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        sources: Vec::new(),
        tags: OrderMap::new(),
        replicas: Vec::new(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
    });
}

#[tokio::test]
async fn update_medium_by_id_fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_update_by_id()
        .times(1)
        .withf(|
            id,
            add_source_ids,
            remove_source_ids,
            add_tag_tag_type_ids,
            remove_tag_tag_type_ids,
            replica_orders,
            created_at,
            tag_depth,
            replicas,
            sources,
        | {
            (
                id,
                add_source_ids,
                remove_source_ids,
                add_tag_tag_type_ids,
                remove_tag_tag_type_ids,
                replica_orders,
                created_at,
                tag_depth,
                replicas,
                sources,
            ) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &[
                    SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                ],
                &[
                    SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                ],
                &[
                    (
                        TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &[
                    (
                        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    ),
                ],
                &[
                    ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                ],
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _, _, _, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_medium_by_id(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        [
            SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ],
        [
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ],
        [
            (
                TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
        [
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ],
        [
            ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ],
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn update_replica_by_id_from_url_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_generate_thumbnail::<SyncIoBridge<BufReader<Cursor<&[_]>>>>()
        .times(1)
        .returning(|_| {
            Box::pin(ok((
                OriginalImage::new("image/jpeg", Size::new(720, 720)),
                ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
            )))
        });

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
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap(),
                    )),
                ),
                Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]),
            )))
        });

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image| {
            (id, thumbnail_image, original_url, original_image) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240))),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(OriginalImage::new("image/jpeg", Size::new(720, 720))),
            )
        })
        .returning(|_, _, _, _| {
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
                mime_type: "image/jpeg".to_string(),
                size: Size::new(720, 720),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Url(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())),
    ).await.unwrap();

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
        mime_type: "image/jpeg".to_string(),
        size: Size::new(720, 720),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
#[serial]
async fn update_replica_by_id_from_content_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_generate_thumbnail::<Cursor<Vec<_>>>()
        .times(1)
        .withf(|read| read == &Cursor::new(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]))
        .returning(|_| {
            Box::pin(ok((
                OriginalImage::new("image/jpeg", Size::new(720, 720)),
                ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
            )))
        });

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_put()
        .times(1)
        .withf(|path, _read, overwrite| {
            (path, overwrite) == (
                &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
                &ObjectOverwriteBehavior::Overwrite,
            )
        })
        .returning(|_, _, _| {
            Box::pin(ok(
                Entry::new(
                    "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                    Some(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())),
                    EntryKind::Object,
                    Some(EntryMetadata::new(
                        4096,
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap(),
                    )),
                ),
            ))
        });

    let mock_objects_repository_scheme = MockObjectsRepository::scheme_context();
    mock_objects_repository_scheme
        .expect()
        .return_const("file");

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image| {
            (id, thumbnail_image, original_url, original_image) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240))),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(OriginalImage::new("image/jpeg", Size::new(720, 720))),
            )
        })
        .returning(|_, _, _, _| {
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
                mime_type: "image/jpeg".to_string(),
                size: Size::new(720, 720),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Content(
            EntryUrlPath::from("/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()),
            vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            MediumOverwriteBehavior::Overwrite,
        ),
    ).await.unwrap();

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
        mime_type: "image/jpeg".to_string(),
        size: Size::new(720, 720),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn update_replica_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_medium_image_processor = MockMediumImageProcessor::new();
    mock_medium_image_processor
        .expect_generate_thumbnail::<SyncIoBridge<BufReader<Cursor<&[_]>>>>()
        .times(1)
        .returning(|_| {
            Box::pin(ok((
                OriginalImage::new("image/jpeg", Size::new(720, 720)),
                ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240)),
            )))
        });

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
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap(),
                        Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap(),
                    )),
                ),
                Cursor::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]),
            )))
        });

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail_image, original_url, original_image| {
            (id, thumbnail_image, original_url, original_image) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(ThumbnailImage::new(vec![0x01, 0x02, 0x03, 0x04], Size::new(240, 240))),
                &Some("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some(OriginalImage::new("image/jpeg", Size::new(720, 720))),
            )
        })
        .returning(|_, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        MediumSource::Url(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn update_source_by_id_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, external_service_id, external_metadata| {
            (id, external_service_id, external_metadata) == (
                &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                &Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
                &Some(ExternalMetadata::Pixiv { id: 56736941 }),
            )
        })
        .returning(|_, _, _| {
            Box::pin(ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            }))
        });

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_source_by_id(
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
        Some(ExternalMetadata::Pixiv { id: 56736941 }),
    ).await.unwrap();

    assert_eq!(actual, Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            slug: "pixiv".to_string(),
            kind: "pixiv".to_string(),
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
        },
        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
    });
}

#[tokio::test]
async fn update_source_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, external_service_id, external_metadata| {
            (id, external_service_id, external_metadata) == (
                &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                &Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
                &Some(ExternalMetadata::Pixiv { id: 56736941 }),
            )
        })
        .returning(|_, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.update_source_by_id(
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
        Some(ExternalMetadata::Pixiv { id: 56736941 }),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn delete_medium_by_id_succeeds() {
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
async fn delete_medium_by_id_with_delete_objects_succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            (ids, tag_depth, replicas, sources) == (
                &[MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))],
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
                            mime_type: "image/png".to_string(),
                            size: Size::new(720, 720),
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
                            mime_type: "image/png".to_string(),
                            size: Size::new(720, 720),
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
async fn delete_medium_by_id_fails() {
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
async fn delete_replica_by_id_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_replica_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")), false).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_replica_by_id_with_delete_object_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_objects_repository = MockObjectsRepository::new();
    mock_objects_repository
        .expect_delete()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids: &[_; 1]| ids == &[ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))])
        .returning(|_| {
            Box::pin(ok(vec![
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
                    mime_type: "image/png".to_string(),
                    size: Size::new(720, 720),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                },
            ]))
        });

    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_replica_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_replica_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_objects_repository = MockObjectsRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor);
    let actual = service.delete_replica_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")), false).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn delete_source_by_id_succeeds() {
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
async fn delete_source_by_id_fails() {
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
