use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use ordermap::OrderMap;
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::{Medium, MediumId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagId},
    },
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
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_create()
        .times(1)
        .withf(|source_ids, created_at, tag_tag_type_ids, tag_depth, sources| {
            source_ids.clone_box().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            ]) &&
            tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
                (
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            (created_at, tag_depth, sources) == (
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
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
                            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
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
                            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
        [
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        [
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
            (
                TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ].into_iter(),
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
                    url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
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
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
async fn fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_create()
        .times(1)
        .withf(|source_ids, created_at, tag_tag_type_ids, tag_depth, sources| {
            source_ids.clone_box().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            ]) &&
            tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
                (
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            (created_at, tag_depth, sources) == (
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
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
        [
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        [
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
            (
                TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ].into_iter(),
        None,
        true,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
