use std::collections::{BTreeMap, BTreeSet};

use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, Thumbnail, ThumbnailId},
        sources::{Source, SourceId},
        tag_types::{TagType, TagTypeId},
        tags::{AliasSet, Tag, TagDepth, TagId},
    },
    repository::{Direction, Order},
    service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
};
use graphql::query::Query;
use indoc::indoc;
use pretty_assertions::assert_eq;
use thumbnails::ThumbnailURLFactory;
use uuid::uuid;

#[tokio::test]
async fn tags_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &Some(TagDepth::new(2, 2)),
                &false,
                &false,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
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
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
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
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
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
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 59).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 3).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allMedia": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": true,
            },
            "edges": [
                {
                    "node": {
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
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "tags": [
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
                        "createdAt": "2022-06-01T12:34:57+00:00",
                        "updatedAt": "2022-06-01T00:05:01+00:00",
                    },
                },
                {
                    "node": {
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
                },
            ],
        },
    }));
}

#[tokio::test]
async fn replicas_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &true,
                &false,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
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
                                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                            }),
                            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                        },
                        Replica {
                            id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                            display_order: 2,
                            thumbnail: Some(Thumbnail {
                                id: ThumbnailId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 4, 0).unwrap(),
                                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 5, 0).unwrap(),
                            }),
                            original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: vec![
                        Replica {
                            id: ReplicaId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                            display_order: 1,
                            thumbnail: None,
                            original_url: "file:///var/lib/hoarder/88888888-8888-8888-8888-888888888888.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                        },
                        Replica {
                            id: ReplicaId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                            display_order: 2,
                            thumbnail: None,
                            original_url: "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 3, 0, 3, 0).unwrap(),
                        },
                    ],
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
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 59).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 3).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(ThumbnailURLFactory::new("https://img.example.com/".to_string()))
        .finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
                        replicas {
                            id
                            displayOrder
                            thumbnail {
                                id
                                url
                                createdAt
                                updatedAt
                            }
                            originalUrl
                            mimeType
                            createdAt
                            updatedAt
                        }
                        createdAt
                        updatedAt
                    }
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allMedia": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": true,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "replicas": [
                            {
                                "id": "66666666-6666-6666-6666-666666666666",
                                "displayOrder": 1,
                                "thumbnail": {
                                    "id": "88888888-8888-8888-8888-888888888888",
                                    "url": "https://img.example.com/88888888-8888-8888-8888-888888888888",
                                    "createdAt": "2022-06-02T00:02:00+00:00",
                                    "updatedAt": "2022-06-02T00:03:00+00:00",
                                },
                                "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                                "mimeType": "image/png",
                                "createdAt": "2022-06-02T00:00:00+00:00",
                                "updatedAt": "2022-06-02T00:01:00+00:00",
                            },
                            {
                                "id": "77777777-7777-7777-7777-777777777777",
                                "displayOrder": 2,
                                "thumbnail": {
                                    "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                                    "url": "https://img.example.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                                    "createdAt": "2022-06-02T00:04:00+00:00",
                                    "updatedAt": "2022-06-02T00:05:00+00:00",
                                },
                                "originalUrl": "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png",
                                "mimeType": "image/png",
                                "createdAt": "2022-06-03T00:02:00+00:00",
                                "updatedAt": "2022-06-03T00:03:00+00:00",
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:56+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "replicas": [
                            {
                                "id": "88888888-8888-8888-8888-888888888888",
                                "displayOrder": 1,
                                "thumbnail": null,
                                "originalUrl": "file:///var/lib/hoarder/88888888-8888-8888-8888-888888888888.png",
                                "mimeType": "image/png",
                                "createdAt": "2022-06-02T00:00:00+00:00",
                                "updatedAt": "2022-06-02T00:01:00+00:00",
                            },
                            {
                                "id": "99999999-9999-9999-9999-999999999999",
                                "displayOrder": 2,
                                "thumbnail": null,
                                "originalUrl": "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.png",
                                "mimeType": "image/png",
                                "createdAt": "2022-06-03T00:02:00+00:00",
                                "updatedAt": "2022-06-03T00:03:00+00:00",
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:57+00:00",
                        "updatedAt": "2022-06-01T00:05:01+00:00",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "replicas": [],
                        "createdAt": "2022-06-01T12:34:58+00:00",
                        "updatedAt": "2022-06-01T00:05:02+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn sources_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &true,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
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
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: vec![
                        Source {
                            id: SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            external_service: ExternalService {
                                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                slug: "pixiv".to_string(),
                                name: "pixiv".to_string(),
                            },
                            external_metadata: ExternalMetadata::Pixiv { id: 1234 },
                            created_at: Utc.with_ymd_and_hms(2016, 5, 5, 7, 6, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2016, 5, 5, 7, 6, 1).unwrap(),
                        },
                    ],
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
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 59).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 3).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allMedia": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": true,
            },
            "edges": [
                {
                    "node": {
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
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "sources": [
                            {
                                "id": "33333333-3333-3333-3333-333333333333",
                                "externalService": {
                                    "id": "11111111-1111-1111-1111-111111111111",
                                    "slug": "pixiv",
                                    "name": "pixiv",
                                },
                                "externalMetadata": {
                                    "pixiv": {
                                        "id": "1234",
                                    },
                                },
                                "createdAt": "2016-05-05T07:06:00+00:00",
                                "updatedAt": "2016-05-05T07:06:01+00:00",
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:57+00:00",
                        "updatedAt": "2022-06-01T00:05:01+00:00",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "sources": [],
                        "createdAt": "2022-06-01T12:34:58+00:00",
                        "updatedAt": "2022-06-01T00:05:02+00:00",
                    },
                },
            ],
        },
    }));
}
