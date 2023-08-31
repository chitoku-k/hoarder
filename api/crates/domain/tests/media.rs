use std::collections::BTreeMap;

use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, ReplicaThumbnail},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagDepth, TagId},
    },
    repository::{
        media::MockMediaRepository,
        replicas::MockReplicasRepository,
        sources::MockSourcesRepository,
        DeleteResult, Direction, Order,
    },
    service::media::{MediaService, MediaServiceInterface},
};
use pretty_assertions::assert_eq;
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
            Ok(Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: vec![
                    Source {
                        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            slug: "twitter".to_string(),
                            name: "Twitter".to_string(),
                        },
                        external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
                        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
                    },
                    Source {
                        id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                            slug: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                        created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
                    },
                ],
                tags: {
                    let mut tags = BTreeMap::new();
                    tags.insert(
                        TagType {
                            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                            slug: "character".to_string(),
                            name: "キャラクター".to_string(),
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
            })
        });

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
                external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            },
            Source {
                id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
            },
        ],
        tags: {
            let mut tags = BTreeMap::new();
            tags.insert(
                TagType {
                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
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
        .returning(|_, _, _, _, _| Err(anyhow!("error creating a medium")));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn create_replica_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail, original_url, mime_type| {
            (medium_id, thumbnail, original_url, mime_type) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(vec![0x01, 0x02, 0x03, 0x04]),
                "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                "image/png",
            )
        })
        .returning(|_, _, _, _| {
            Ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: Some(1),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            })
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        Some(vec![0x01, 0x02, 0x03, 0x04]),
        "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
        "image/png",
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: Some(1),
        has_thumbnail: true,
        original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: "image/png".to_string(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn create_replica_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_create()
        .times(1)
        .withf(|medium_id, thumbnail, original_url, mime_type| {
            (medium_id, thumbnail, original_url, mime_type) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(vec![0x01, 0x02, 0x03, 0x04]),
                "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                "image/png",
            )
        })
        .returning(|_, _, _, _| Err(anyhow!("error creating a replica")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.create_replica(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        Some(vec![0x01, 0x02, 0x03, 0x04]),
        "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
        "image/png",
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn create_source_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_create()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &ExternalMetadata::Twitter { id: 727620202049900544 },
            )
        })
        .returning(|_, _| {
            Ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
                external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            })
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.create_source(
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ExternalMetadata::Twitter { id: 727620202049900544 },
    ).await.unwrap();

    assert_eq!(actual, Source {
        id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            slug: "twitter".to_string(),
            name: "Twitter".to_string(),
        },
        external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
    });
}

#[tokio::test]
async fn create_source_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_create()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &ExternalMetadata::Twitter { id: 727620202049900544 },
            )
        })
        .returning(|_, _| Err(anyhow!("error creating a source")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.create_source(
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ExternalMetadata::Twitter { id: 727620202049900544 },
    ).await;

    assert!(actual.is_err());
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
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
            ])
        });

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
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
        .returning(|_, _, _, _, _, _, _| Err(anyhow!("error fetching media")));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_media(
        Some(TagDepth::new(1, 1)),
        true,
        true,
        Some((Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
        Order::Ascending,
        Direction::Forward,
        10
    ).await;

    assert!(actual.is_err());
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
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
            ])
        });

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
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
        .returning(|_, _, _, _| Err(anyhow!("error fetching the media")));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_media_by_ids(
        vec![
            MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
        ],
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await;

    assert!(actual.is_err());
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
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
            ])
        });

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
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
        .returning(|_, _, _, _, _, _, _, _| Err(anyhow!("error fetching the media")));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
    ).await;

    assert!(actual.is_err());
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
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
            ])
        });

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
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
        .returning(|_, _, _, _, _, _, _, _| Err(anyhow!("error fetching the media")));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_replicas_by_ids_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

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
            Ok(vec![
                Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: Some(1),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                },
                Replica {
                    id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    display_order: Some(2),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                },
            ])
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_replicas_by_ids(vec![
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
    ]).await.unwrap();

    assert_eq!(actual, vec![
        Replica {
            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            display_order: Some(1),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            display_order: Some(2),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn get_replicas_by_ids_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

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
        .returning(|_| Err(anyhow!("error fetching the replicas")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_replicas_by_ids(vec![
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
    ]).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_replica_by_original_url_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
        .returning(|_| {
            Ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: Some(1),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            })
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_replica_by_original_url("file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png").await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: Some(1),
        has_thumbnail: true,
        original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: "image/png".to_string(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn get_replica_by_original_url_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Err(anyhow!("error fetching the replica")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_replica_by_original_url("file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png").await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_source_by_external_metadata_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

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
            Ok(Source {
                id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
            })
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_source_by_external_metadata(
         ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
         ExternalMetadata::Pixiv { id: 56736941 },
    ).await.unwrap();

    assert_eq!(actual, Source {
        id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            slug: "pixiv".to_string(),
            name: "pixiv".to_string(),
        },
        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
        created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
    });
}

#[tokio::test]
async fn get_sources_by_external_metadata_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

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
        .returning(|_, _| Err(anyhow!("error fetching the sources")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_source_by_external_metadata(
         ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
         ExternalMetadata::Pixiv { id: 56736941 },
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_thumbnail_by_id_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_thumbnail_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| {
            Ok(ReplicaThumbnail {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: Some(1),
                thumbnail: Some(vec![0x01, 0x02, 0x03, 0x04]),
                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            })
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_thumbnail_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await.unwrap();

    assert_eq!(actual, ReplicaThumbnail {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: Some(1),
        thumbnail: Some(vec![0x01, 0x02, 0x03, 0x04]),
        original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
        mime_type: "image/png".to_string(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn get_thumbnail_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_fetch_thumbnail_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Err(anyhow!("error fetching the replica")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.get_thumbnail_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await;

    assert!(actual.is_err());
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
            Ok(Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
            })
        });

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
        tags: BTreeMap::new(),
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
        .returning(|_, _, _, _, _, _, _, _, _, _| Err(anyhow!("error updating the medium")));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn update_replica_by_id_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail, original_url, mime_type| {
            (id, thumbnail, original_url, mime_type) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(vec![0x01, 0x02, 0x03, 0x04]),
                &Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some("image/jpeg"),
            )
        })
        .returning(|_, _, _, _| {
            Ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: Some(1),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                mime_type: "image/jpeg".to_string(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            })
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        Some(vec![0x01, 0x02, 0x03, 0x04]),
        Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
        Some("image/jpeg"),
    ).await.unwrap();

    assert_eq!(actual, Replica {
        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        display_order: Some(1),
        has_thumbnail: true,
        original_url: "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
        mime_type: "image/jpeg".to_string(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn update_replica_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, thumbnail, original_url, mime_type| {
            (id, thumbnail, original_url, mime_type) == (
                &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                &Some(vec![0x01, 0x02, 0x03, 0x04]),
                &Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                &Some("image/jpeg"),
            )
        })
        .returning(|_, _, _, _| Err(anyhow!("error updating the replica")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.update_replica_by_id(
        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        Some(vec![0x01, 0x02, 0x03, 0x04]),
        Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
        Some("image/jpeg"),
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn update_source_by_id_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

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
            Ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            })
        });

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
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
            name: "pixiv".to_string(),
        },
        external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
        created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
    });
}

#[tokio::test]
async fn update_source_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

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
        .returning(|_, _, _| Err(anyhow!("error updating the source")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.update_source_by_id(
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
        Some(ExternalMetadata::Pixiv { id: 56736941 }),
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn delete_medium_by_id_succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")))
        .returning(|_| Ok(DeleteResult::Deleted(1)));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_medium_by_id_fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")))
        .returning(|_| Err(anyhow!("error deleting the medium")));

    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn delete_replica_by_id_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Ok(DeleteResult::Deleted(1)));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.delete_replica_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_replica_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();

    let mut mock_replicas_repository = MockReplicasRepository::new();
    mock_replicas_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
        .returning(|_| Err(anyhow!("error deleting the replica")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.delete_replica_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn delete_source_by_id_succeeds() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Ok(DeleteResult::Deleted(1)));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.delete_source_by_id(SourceId::from(uuid!("11111111-1111-1111-1111-111111111111"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_source_by_id_fails() {
    let mock_media_repository = MockMediaRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();

    let mut mock_sources_repository = MockSourcesRepository::new();
    mock_sources_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Err(anyhow!("error deleting the source")));

    let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
    let actual = service.delete_source_by_id(SourceId::from(uuid!("11111111-1111-1111-1111-111111111111"))).await;

    assert!(actual.is_err());
}
