use std::{collections::{BTreeMap, BTreeSet}, sync::Arc};

use application::service::{
    media::{MediaURLFactoryInterface, MockMediaURLFactoryInterface},
    thumbnails::{MockThumbnailURLFactoryInterface, ThumbnailURLFactoryInterface},
};
use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalService, ExternalServiceId, ExternalMetadata},
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, Size, Thumbnail, ThumbnailId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{Tag, AliasSet, TagDepth, TagId},
    },
    service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
};
use graphql::query::Query;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::{uuid, Uuid};

// Concrete type is required both in implementation and expectation.
type IntoIterMap<T, U> = std::iter::Map<std::vec::IntoIter<T>, fn(T) -> U>;

#[tokio::test]
async fn tags_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ]) &&
            (tag_depth, replicas, sources) == (
                &Some(TagDepth::new(2, 2)),
                &false,
                &false,
            )
        })
        .returning(|_, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
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
                                    aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                },
                                Tag {
                                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: Default::default(),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                                },
                            ],
                        );
                        tags.insert(
                            TagType {
                                id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                slug: "work".to_string(),
                                name: "作品".to_string(),
                            },
                            vec![
                                Tag {
                                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: vec![
                                        Tag {
                                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                            name: "赤座あかり".to_string(),
                                            kana: "あかざあかり".to_string(),
                                            aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                        },
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
                                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
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
                                    aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                    parent: Some(Box::new(Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                    })),
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
                            },
                            vec![
                                Tag {
                                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: vec![
                                        Tag {
                                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                            name: "赤座あかり".to_string(),
                                            kana: "あかざあかり".to_string(),
                                            aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                        },
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
                                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                id
                tags {
                    tag {
                        id
                        name
                        kana
                        aliases
                        parent {
                            id
                            name
                            kana
                            aliases
                            parent {
                                id
                                name
                                kana
                                aliases
                                createdAt
                                updatedAt
                            }
                            createdAt
                            updatedAt
                        }
                        children {
                            id
                            name
                            kana
                            aliases
                            children {
                                id
                                name
                                kana
                                aliases
                                createdAt
                                updatedAt
                            }
                            createdAt
                            updatedAt
                        }
                        createdAt
                        updatedAt
                    }
                    type {
                        id
                        slug
                        name
                    }
                }
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "media": [
            {
                "id": "77777777-7777-7777-7777-777777777777",
                "tags": [
                    {
                        "tag": {
                            "id": "33333333-3333-3333-3333-333333333333",
                            "name": "赤座あかり",
                            "kana": "あかざあかり",
                            "aliases": ["アッカリーン"],
                            "parent": {
                                "id": "22222222-2222-2222-2222-222222222222",
                                "name": "ゆるゆり",
                                "kana": "ゆるゆり",
                                "aliases": [],
                                "parent": null,
                                "createdAt": "2022-06-01T00:00:00+00:00",
                                "updatedAt": "2022-06-01T00:01:00+00:00",
                            },
                            "children": [],
                            "createdAt": "2022-06-01T00:00:00+00:00",
                            "updatedAt": "2022-06-01T00:01:00+00:00",
                        },
                        "type": {
                            "id": "44444444-4444-4444-4444-444444444444",
                            "slug": "character",
                            "name": "キャラクター",
                        },
                    },
                    {
                        "tag": {
                            "id": "55555555-5555-5555-5555-555555555555",
                            "name": "歳納京子",
                            "kana": "としのうきょうこ",
                            "aliases": [],
                            "parent": {
                                "id": "22222222-2222-2222-2222-222222222222",
                                "name": "ゆるゆり",
                                "kana": "ゆるゆり",
                                "aliases": [],
                                "parent": null,
                                "createdAt": "2022-06-01T00:00:00+00:00",
                                "updatedAt": "2022-06-01T00:01:00+00:00",
                            },
                            "children": [],
                            "createdAt": "2022-06-01T00:02:00+00:00",
                            "updatedAt": "2022-06-01T00:03:00+00:00",
                        },
                        "type": {
                            "id": "44444444-4444-4444-4444-444444444444",
                            "slug": "character",
                            "name": "キャラクター",
                        },
                    },
                    {
                        "tag": {
                            "id": "22222222-2222-2222-2222-222222222222",
                            "name": "ゆるゆり",
                            "kana": "ゆるゆり",
                            "aliases": [],
                            "parent": null,
                            "children": [
                                {
                                    "id": "33333333-3333-3333-3333-333333333333",
                                    "name": "赤座あかり",
                                    "kana": "あかざあかり",
                                    "aliases": ["アッカリーン"],
                                    "children": [],
                                    "createdAt": "2022-06-01T00:00:00+00:00",
                                    "updatedAt": "2022-06-01T00:01:00+00:00",
                                },
                                {
                                    "id": "55555555-5555-5555-5555-555555555555",
                                    "name": "歳納京子",
                                    "kana": "としのうきょうこ",
                                    "aliases": [],
                                    "children": [],
                                    "createdAt": "2022-06-01T00:02:00+00:00",
                                    "updatedAt": "2022-06-01T00:03:00+00:00",
                                },
                            ],
                            "createdAt": "2022-06-01T00:00:00+00:00",
                            "updatedAt": "2022-06-01T00:01:00+00:00",
                        },
                        "type": {
                            "id": "66666666-6666-6666-6666-666666666666",
                            "slug": "work",
                            "name": "作品",
                        },
                    },
                ],
                "createdAt": "2022-06-01T12:34:56+00:00",
                "updatedAt": "2022-06-01T00:05:00+00:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "tags": [
                    {
                        "tag": {
                            "id": "33333333-3333-3333-3333-333333333333",
                            "name": "赤座あかり",
                            "kana": "あかざあかり",
                            "aliases": ["アッカリーン"],
                            "parent": {
                                "id": "22222222-2222-2222-2222-222222222222",
                                "name": "ゆるゆり",
                                "kana": "ゆるゆり",
                                "aliases": [],
                                "parent": null,
                                "createdAt": "2022-06-01T00:00:00+00:00",
                                "updatedAt": "2022-06-01T00:01:00+00:00",
                            },
                            "children": [],
                            "createdAt": "2022-06-01T00:00:00+00:00",
                            "updatedAt": "2022-06-01T00:01:00+00:00",
                        },
                        "type": {
                            "id": "44444444-4444-4444-4444-444444444444",
                            "slug": "character",
                            "name": "キャラクター",
                        },
                    },
                    {
                        "tag": {
                            "id": "22222222-2222-2222-2222-222222222222",
                            "name": "ゆるゆり",
                            "kana": "ゆるゆり",
                            "aliases": [],
                            "parent": null,
                            "children": [
                                {
                                    "id": "33333333-3333-3333-3333-333333333333",
                                    "name": "赤座あかり",
                                    "kana": "あかざあかり",
                                    "aliases": ["アッカリーン"],
                                    "children": [],
                                    "createdAt": "2022-06-01T00:00:00+00:00",
                                    "updatedAt": "2022-06-01T00:01:00+00:00",
                                },
                                {
                                    "id": "55555555-5555-5555-5555-555555555555",
                                    "name": "歳納京子",
                                    "kana": "としのうきょうこ",
                                    "aliases": [],
                                    "children": [],
                                    "createdAt": "2022-06-01T00:02:00+00:00",
                                    "updatedAt": "2022-06-01T00:03:00+00:00",
                                },
                            ],
                            "createdAt": "2022-06-01T00:00:00+00:00",
                            "updatedAt": "2022-06-01T00:01:00+00:00",
                        },
                        "type": {
                            "id": "66666666-6666-6666-6666-666666666666",
                            "slug": "work",
                            "name": "作品",
                        },
                    },
                ],
                "createdAt": "2022-06-01T12:34:58+00:00",
                "updatedAt": "2022-06-01T00:05:02+00:00",
            },
        ],
    }));
}

#[tokio::test]
async fn replicas_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &true,
                &false,
            )
        })
        .returning(|_, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
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
                            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
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
                            original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                            mime_type: "image/png".to_string(),
                            size: Size::new(720, 720),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                        },
                    ],
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

    let tags_service = MockTagsServiceInterface::new();

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_rewrite_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
        .returning(|_| "https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string());

    media_url_factory
        .expect_rewrite_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png")
        .returning(|_| "https://original.example.com/99999999-9999-9999-9999-999999999999.png".to_string());

    let mut thumbnail_url_factory = MockThumbnailURLFactoryInterface::new();
    thumbnail_url_factory
        .expect_get()
        .times(1)
        .withf(|thumbnail_id| thumbnail_id == &ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")))
        .returning(|_| "https://img.example.com/88888888-8888-8888-8888-888888888888".to_string());

    thumbnail_url_factory
        .expect_get()
        .times(1)
        .withf(|thumbnail_id| thumbnail_id == &ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")))
        .returning(|_| "https://img.example.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string());

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .data::<Arc<dyn ThumbnailURLFactoryInterface>>(Arc::new(thumbnail_url_factory))
        .finish();

    let req = indoc! {r#"
        query {
            media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                id
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
        "media": [
            {
                "id": "77777777-7777-7777-7777-777777777777",
                "replicas": [
                    {
                        "id": "66666666-6666-6666-6666-666666666666",
                        "displayOrder": 1,
                        "thumbnail": {
                            "id": "88888888-8888-8888-8888-888888888888",
                            "url": "https://img.example.com/88888888-8888-8888-8888-888888888888",
                            "width": 240,
                            "height": 240,
                            "createdAt": "2022-06-02T00:02:00+00:00",
                            "updatedAt": "2022-06-02T00:03:00+00:00",
                        },
                        "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
                        "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                        "mimeType": "image/png",
                        "width": 720,
                        "height": 720,
                        "createdAt": "2022-06-02T00:00:00+00:00",
                        "updatedAt": "2022-06-02T00:01:00+00:00",
                    },
                    {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "displayOrder": 2,
                        "thumbnail": {
                            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "url": "https://img.example.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "width": 240,
                            "height": 240,
                            "createdAt": "2022-06-02T00:04:00+00:00",
                            "updatedAt": "2022-06-02T00:05:00+00:00",
                        },
                        "url": "https://original.example.com/99999999-9999-9999-9999-999999999999.png",
                        "originalUrl": "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png",
                        "mimeType": "image/png",
                        "width": 720,
                        "height": 720,
                        "createdAt": "2022-06-03T00:02:00+00:00",
                        "updatedAt": "2022-06-03T00:03:00+00:00",
                    },
                ],
                "createdAt": "2022-06-01T12:34:56+00:00",
                "updatedAt": "2022-06-01T00:05:00+00:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "replicas": [],
                "createdAt": "2022-06-01T12:34:58+00:00",
                "updatedAt": "2022-06-01T00:05:02+00:00",
            },
        ],
    }));
}

#[tokio::test]
async fn sources_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &false,
                &true,
            )
        })
        .returning(|_, _, _, _| {
            Ok(vec![
                Medium {
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

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                id
                sources {
                    id
                    externalService {
                        id
                        slug
                        name
                    }
                    externalMetadata
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
        "media": [
            {
                "id": "77777777-7777-7777-7777-777777777777",
                "sources": [
                    {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "externalService": {
                            "id": "33333333-3333-3333-3333-333333333333",
                            "slug": "twitter",
                            "name": "Twitter",
                        },
                        "externalMetadata": {
                            "twitter": {
                                "id": "727620202049900544",
                            },
                        },
                        "createdAt": "2016-05-04T07:05:00+00:00",
                        "updatedAt": "2016-05-04T07:05:01+00:00",
                    },
                    {
                        "id": "22222222-2222-2222-2222-222222222222",
                        "externalService": {
                            "id": "11111111-1111-1111-1111-111111111111",
                            "slug": "pixiv",
                            "name": "pixiv",
                        },
                        "externalMetadata": {
                            "pixiv": {
                                "id": "56736941",
                            },
                        },
                        "createdAt": "2016-05-06T05:14:00+00:00",
                        "updatedAt": "2016-05-06T05:14:01+00:00",
                    },
                ],
                "createdAt": "2022-06-01T12:34:56+00:00",
                "updatedAt": "2022-06-01T00:05:00+00:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "sources": [],
                "createdAt": "2022-06-01T12:34:58+00:00",
                "updatedAt": "2022-06-01T00:05:02+00:00",
            },
        ],
    }));
}
