use std::sync::Arc;

use application::service::{media::MediaURLFactoryInterface, thumbnails::ThumbnailURLFactoryInterface};
use async_graphql::{Schema, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::entity::{
    external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
    media::{Medium, MediumId},
    replicas::{Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId},
    sources::{Source, SourceId},
    tag_types::{TagType, TagTypeId},
    tags::{Tag, TagDepth, TagId},
};
use futures::future::ok;
use indoc::indoc;
use ordermap::OrderMap;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::{mutation::Mutation, query::Query};

use super::mocks::{
    application::service::{media::MockMediaURLFactoryInterface, thumbnails::MockThumbnailURLFactoryInterface},
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
};

#[tokio::test]
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_update_medium_by_id()
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
            add_source_ids.clone_box().eq([
                SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ]) &&
            remove_source_ids.clone_box().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ]) &&
            add_tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            remove_tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            replica_orders.clone_box().eq([
                ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            ]) &&
            (id, created_at, tag_depth, replicas, sources) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &Some(TagDepth::new(0, 0)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _, _, _, _, _, _, _| {
            Box::pin(ok(Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: vec![
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
                    Source {
                        id: SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        external_service: ExternalService {
                            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                            slug: "pixiv".to_string(),
                            kind: "pixiv".to_string(),
                            name: "pixiv".to_string(),
                            base_url: Some("https://www.pixiv.net".to_string()),
                            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                        },
                        external_metadata: ExternalMetadata::Pixiv { id: 1234 },
                        created_at: Utc.with_ymd_and_hms(2016, 5, 5, 7, 6, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2016, 5, 5, 7, 6, 1).unwrap(),
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
                                id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: Default::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
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
                            id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                            slug: "work".to_string(),
                            name: "作品".to_string(),
                            kana: "さくひん".to_string(),
                        },
                        vec![
                            Tag {
                                id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                name: "ゆるゆり".to_string(),
                                kana: "ゆるゆり".to_string(),
                                aliases: Default::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                            },
                        ],
                    );
                    tags
                },
                replicas: vec![
                    Replica {
                        id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        display_order: 1,
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
                    Replica {
                        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                        display_order: 2,
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
                ],
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
            }))
        });

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///99999999-9999-9999-9999-999999999999.png")
        .returning(|_| Some("https://original.example.com/99999999-9999-9999-9999-999999999999.png".to_string()));

    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Some("https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string()));

    let mut thumbnail_url_factory = MockThumbnailURLFactoryInterface::new();
    thumbnail_url_factory
        .expect_get()
        .times(1)
        .withf(|thumbnail_id| thumbnail_id == &ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")))
        .returning(|_| "https://img.example.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string());

    thumbnail_url_factory
        .expect_get()
        .times(1)
        .withf(|thumbnail_id| thumbnail_id == &ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")))
        .returning(|_| "https://img.example.com/88888888-8888-8888-8888-888888888888".to_string());

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .data::<Arc<dyn ThumbnailURLFactoryInterface>>(Arc::new(thumbnail_url_factory))
        .finish();

    let req = indoc! {r#"
        mutation {
            updateMedium(
                id: "77777777-7777-7777-7777-777777777777",
                addSourceIds: [
                    "33333333-3333-3333-3333-333333333333",
                ],
                removeSourceIds: [
                    "11111111-1111-1111-1111-111111111111",
                ],
                addTagIds: [
                    {
                        tagId: "22222222-2222-2222-2222-222222222222",
                        tagTypeId: "44444444-4444-4444-4444-444444444444",
                    },
                ],
                removeTagIds: [
                    {
                        tagId: "33333333-3333-3333-3333-333333333333",
                        tagTypeId: "44444444-4444-4444-4444-444444444444",
                    },
                ],
                replicaOrders: [
                    "77777777-7777-7777-7777-777777777777",
                    "66666666-6666-6666-6666-666666666666",
                ],
                createdAt: "2022-06-01T12:34:56+00:00",
            ) {
                id
                sources {
                    id
                    externalService {
                        id
                        slug
                        kind
                        name
                        baseUrl
                        urlPattern
                    }
                    externalMetadata
                    url
                    createdAt
                    updatedAt
                }
                tags {
                    tag {
                        id
                        name
                        kana
                        aliases
                        createdAt
                        updatedAt
                    }
                    type {
                        id
                        slug
                        name
                        kana
                    }
                }
                replicas {
                    id
                    displayOrder
                    thumbnail {
                        id
                        url
                        width
                        height
                        createdAt
                        updatedAt
                    }
                    url
                    originalUrl
                    mimeType
                    width
                    height
                    createdAt
                    updatedAt
                }
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "updateMedium": {
            "id": "77777777-7777-7777-7777-777777777777",
            "sources": [
                {
                    "id": "22222222-2222-2222-2222-222222222222",
                    "externalService": {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "slug": "pixiv",
                        "kind": "pixiv",
                        "name": "pixiv",
                        "baseUrl": "https://www.pixiv.net",
                        "urlPattern": r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
                    },
                    "externalMetadata": {
                        "pixiv": {
                            "id": "56736941",
                        },
                    },
                    "url": "https://www.pixiv.net/artworks/56736941",
                    "createdAt": "2016-05-06T05:14:00+00:00",
                    "updatedAt": "2016-05-06T05:14:01+00:00",
                },
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "externalService": {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "slug": "pixiv",
                        "kind": "pixiv",
                        "name": "pixiv",
                        "baseUrl": "https://www.pixiv.net",
                        "urlPattern": r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
                    },
                    "externalMetadata": {
                        "pixiv": {
                            "id": "1234",
                        },
                    },
                    "url": "https://www.pixiv.net/artworks/1234",
                    "createdAt": "2016-05-05T07:06:00+00:00",
                    "updatedAt": "2016-05-05T07:06:01+00:00",
                },
            ],
            "tags": [
                {
                    "tag": {
                        "id": "55555555-5555-5555-5555-555555555555",
                        "name": "歳納京子",
                        "kana": "としのうきょうこ",
                        "aliases": [],
                        "createdAt": "2022-06-01T00:02:00+00:00",
                        "updatedAt": "2022-06-01T00:03:00+00:00",
                    },
                    "type": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "slug": "character",
                        "name": "キャラクター",
                        "kana": "キャラクター",
                    },
                },
                {
                    "tag": {
                        "id": "22222222-2222-2222-2222-222222222222",
                        "name": "ゆるゆり",
                        "kana": "ゆるゆり",
                        "aliases": [],
                        "createdAt": "2022-06-01T00:00:00+00:00",
                        "updatedAt": "2022-06-01T00:01:00+00:00",
                    },
                    "type": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "slug": "character",
                        "name": "キャラクター",
                        "kana": "キャラクター",
                    },
                },
                {
                    "tag": {
                        "id": "22222222-2222-2222-2222-222222222222",
                        "name": "ゆるゆり",
                        "kana": "ゆるゆり",
                        "aliases": [],
                        "createdAt": "2022-06-01T00:00:00+00:00",
                        "updatedAt": "2022-06-01T00:01:00+00:00",
                    },
                    "type": {
                        "id": "66666666-6666-6666-6666-666666666666",
                        "slug": "work",
                        "name": "作品",
                        "kana": "さくひん",
                    },
                },
            ],
            "replicas": [
                {
                    "id": "77777777-7777-7777-7777-777777777777",
                    "displayOrder": 1,
                    "thumbnail": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "url": "https://img.example.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "width": 240,
                        "height": 240,
                        "createdAt": "2022-06-02T00:04:00+00:00",
                        "updatedAt": "2022-06-02T00:05:00+00:00",
                    },
                    "url": "https://original.example.com/99999999-9999-9999-9999-999999999999.png",
                    "originalUrl": "file:///99999999-9999-9999-9999-999999999999.png",
                    "mimeType": "image/png",
                    "width": 720,
                    "height": 720,
                    "createdAt": "2022-06-03T00:02:00+00:00",
                    "updatedAt": "2022-06-03T00:03:00+00:00",
                },
                {
                    "id": "66666666-6666-6666-6666-666666666666",
                    "displayOrder": 2,
                    "thumbnail": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "url": "https://img.example.com/88888888-8888-8888-8888-888888888888",
                        "width": 240,
                        "height": 240,
                        "createdAt": "2022-06-02T00:02:00+00:00",
                        "updatedAt": "2022-06-02T00:03:00+00:00",
                    },
                    "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
                    "originalUrl": "file:///77777777-7777-7777-7777-777777777777.png",
                    "mimeType": "image/png",
                    "width": 720,
                    "height": 720,
                    "createdAt": "2022-06-02T00:00:00+00:00",
                    "updatedAt": "2022-06-02T00:01:00+00:00",
                },
            ],
            "createdAt": "2022-06-01T12:34:56+00:00",
            "updatedAt": "2022-06-01T00:05:00+00:00",
        },
    }));
}
