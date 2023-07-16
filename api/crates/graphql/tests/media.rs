use std::collections::{BTreeMap, BTreeSet};

use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::{
    entity::{
        external_services::{self, ExternalServiceId},
        media::{self, MediumId},
        replicas::{self, ReplicaId},
        sources::{self, SourceId},
        tag_types::{self, TagTypeId},
        tags::{self, AliasSet, TagDepth, TagId},
    },
    repository::OrderDirection,
    service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
};
use chrono::NaiveDate;
use graphql::{query::Query, tags::TagTagTypeInput};
use indoc::indoc;
use pretty_assertions::assert_eq;
use thumbnails::ThumbnailURLFactory;
use uuid::{uuid, Uuid};

// Concrete type is required both in implementation and expectation.
type IntoIterMap<T, U> = std::iter::Map<std::vec::IntoIter<T>, fn(T) -> U>;

#[tokio::test]
async fn all_media_with_tags_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &Some(TagDepth::new(2, 2)),
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: {
                        let mut tags = BTreeMap::new();
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                slug: "character".to_string(),
                                name: "キャラクター".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                    name: "赤座あかり".to_string(),
                                    kana: "あかざあかり".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                    parent: Some(Box::new(tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                                tags::Tag {
                                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: Default::default(),
                                    parent: Some(Box::new(tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                },
                            ],
                        );
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                slug: "work".to_string(),
                                name: "作品".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: vec![
                                        tags::Tag {
                                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                            name: "赤座あかり".to_string(),
                                            kana: "あかざあかり".to_string(),
                                            aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        },
                                        tags::Tag {
                                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: {
                        let mut tags = BTreeMap::new();
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                slug: "character".to_string(),
                                name: "キャラクター".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: Default::default(),
                                    parent: Some(Box::new(tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                },
                            ],
                        );
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                slug: "work".to_string(),
                                name: "作品".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: vec![
                                        tags::Tag {
                                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                            name: "赤座あかり".to_string(),
                                            kana: "あかざあかり".to_string(),
                                            aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        },
                                        tags::Tag {
                                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: {
                        let mut tags = BTreeMap::new();
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                slug: "character".to_string(),
                                name: "キャラクター".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                    name: "赤座あかり".to_string(),
                                    kana: "あかざあかり".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                    parent: Some(Box::new(tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                slug: "work".to_string(),
                                name: "作品".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: vec![
                                        tags::Tag {
                                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                            name: "赤座あかり".to_string(),
                                            kana: "あかざあかり".to_string(),
                                            aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        },
                                        tags::Tag {
                                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
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
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "children": [],
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
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
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "children": [],
                                    "createdAt": "2022-06-01T00:02:00",
                                    "updatedAt": "2022-06-01T00:03:00",
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
                                            "createdAt": "2022-06-01T00:00:00",
                                            "updatedAt": "2022-06-01T00:01:00",
                                        },
                                        {
                                            "id": "55555555-5555-5555-5555-555555555555",
                                            "name": "歳納京子",
                                            "kana": "としのうきょうこ",
                                            "aliases": [],
                                            "children": [],
                                            "createdAt": "2022-06-01T00:02:00",
                                            "updatedAt": "2022-06-01T00:03:00",
                                        },
                                    ],
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                "type": {
                                    "id": "66666666-6666-6666-6666-666666666666",
                                    "slug": "work",
                                    "name": "作品",
                                },
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
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
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "children": [],
                                    "createdAt": "2022-06-01T00:02:00",
                                    "updatedAt": "2022-06-01T00:03:00",
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
                                            "createdAt": "2022-06-01T00:00:00",
                                            "updatedAt": "2022-06-01T00:01:00",
                                        },
                                        {
                                            "id": "55555555-5555-5555-5555-555555555555",
                                            "name": "歳納京子",
                                            "kana": "としのうきょうこ",
                                            "aliases": [],
                                            "children": [],
                                            "createdAt": "2022-06-01T00:02:00",
                                            "updatedAt": "2022-06-01T00:03:00",
                                        },
                                    ],
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                "type": {
                                    "id": "66666666-6666-6666-6666-666666666666",
                                    "slug": "work",
                                    "name": "作品",
                                },
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
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
                                        "createdAt": "2022-06-01T00:00:00",
                                        "updatedAt": "2022-06-01T00:01:00",
                                    },
                                    "children": [],
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
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
                                            "createdAt": "2022-06-01T00:00:00",
                                            "updatedAt": "2022-06-01T00:01:00",
                                        },
                                        {
                                            "id": "55555555-5555-5555-5555-555555555555",
                                            "name": "歳納京子",
                                            "kana": "としのうきょうこ",
                                            "aliases": [],
                                            "children": [],
                                            "createdAt": "2022-06-01T00:02:00",
                                            "updatedAt": "2022-06-01T00:03:00",
                                        },
                                    ],
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                "type": {
                                    "id": "66666666-6666-6666-6666-666666666666",
                                    "slug": "work",
                                    "name": "作品",
                                },
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_with_replicas_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &true,
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: vec![
                        replicas::Replica {
                            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                            display_order: Some(1),
                            has_thumbnail: true,
                            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                        },
                        replicas::Replica {
                            id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                            display_order: Some(2),
                            has_thumbnail: true,
                            original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: vec![
                        replicas::Replica {
                            id: ReplicaId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                            display_order: Some(1),
                            has_thumbnail: false,
                            original_url: "file:///var/lib/hoarder/88888888-8888-8888-8888-888888888888.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                        },
                        replicas::Replica {
                            id: ReplicaId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                            display_order: Some(2),
                            has_thumbnail: false,
                            original_url: "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
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
                            originalUrl
                            thumbnailUrl
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
                                "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                                "thumbnailUrl": "https://img.example.com/66666666-6666-6666-6666-666666666666",
                                "mimeType": "image/png",
                                "createdAt": "2022-06-02T00:00:00",
                                "updatedAt": "2022-06-02T00:01:00",
                            },
                            {
                                "id": "77777777-7777-7777-7777-777777777777",
                                "displayOrder": 2,
                                "originalUrl": "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png",
                                "thumbnailUrl": "https://img.example.com/77777777-7777-7777-7777-777777777777",
                                "mimeType": "image/png",
                                "createdAt": "2022-06-03T00:02:00",
                                "updatedAt": "2022-06-03T00:03:00",
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "replicas": [
                            {
                                "id": "88888888-8888-8888-8888-888888888888",
                                "displayOrder": 1,
                                "originalUrl": "file:///var/lib/hoarder/88888888-8888-8888-8888-888888888888.png",
                                "thumbnailUrl": null,
                                "mimeType": "image/png",
                                "createdAt": "2022-06-02T00:00:00",
                                "updatedAt": "2022-06-02T00:01:00",
                            },
                            {
                                "id": "99999999-9999-9999-9999-999999999999",
                                "displayOrder": 2,
                                "originalUrl": "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.png",
                                "thumbnailUrl": null,
                                "mimeType": "image/png",
                                "createdAt": "2022-06-03T00:02:00",
                                "updatedAt": "2022-06-03T00:03:00",
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "replicas": [],
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_with_sources_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &true,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: vec![
                        sources::Source {
                            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                            external_service: external_services::ExternalService {
                                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                slug: "twitter".to_string(),
                                name: "Twitter".to_string(),
                            },
                            external_metadata: external_services::ExternalMetadata::Twitter { id: 727620202049900544 },
                            created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                        },
                        sources::Source {
                            id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                            external_service: external_services::ExternalService {
                                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                slug: "pixiv".to_string(),
                                name: "pixiv".to_string(),
                            },
                            external_metadata: external_services::ExternalMetadata::Pixiv { id: 56736941 },
                            created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
                        },
                    ],
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: vec![
                        sources::Source {
                            id: SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            external_service: external_services::ExternalService {
                                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                slug: "pixiv".to_string(),
                                name: "pixiv".to_string(),
                            },
                            external_metadata: external_services::ExternalMetadata::Pixiv { id: 1234 },
                            created_at: NaiveDate::from_ymd_opt(2016, 5, 5).and_then(|d| d.and_hms_opt(7, 6, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2016, 5, 5).and_then(|d| d.and_hms_opt(7, 6, 1)).unwrap(),
                        },
                    ],
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
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
                                "createdAt": "2016-05-04T07:05:00",
                                "updatedAt": "2016-05-04T07:05:01",
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
                                "createdAt": "2016-05-06T05:14:00",
                                "updatedAt": "2016-05-06T05:14:01",
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
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
                                "createdAt": "2016-05-05T07:06:00",
                                "updatedAt": "2016-05-05T07:06:01",
                            },
                        ],
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "sources": [],
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_first_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
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
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_first_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, order: DESC) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_first_after_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, after: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_first_after_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &None,
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, order: DESC, after: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_first_before_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, before: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_first_before_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, order: DESC, before: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_last_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(last: 3, order: ASC) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasPreviousPage": true,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_last_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(last: 3, order: DESC) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasPreviousPage": true,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_last_after_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &None,
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(last: 3, after: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_last_after_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(last: 3, order: DESC, after: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_last_before_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(last: 3, before: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_last_before_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(last: 3, order: DESC, before: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_by_source_ids_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_source_ids::<IntoIterMap<Uuid, SourceId>>()
        .times(1)
        .withf(|source_ids, tag_depth, replicas, sources, since, until, order, limit| {
            source_ids.clone().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ]) &&
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, sourceIds: ["11111111-1111-1111-1111-111111111111", "33333333-3333-3333-3333-333333333333"]) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn all_media_by_tag_ids_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_tag_ids::<IntoIterMap<TagTagTypeInput, (TagId, TagTypeId)>>()
        .times(1)
        .withf(|tag_ids, tag_depth, replicas, sources, since, until, order, limit| {
            tag_ids.clone().eq([
                (
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(
                first: 3,
                tagIds: [
                    {
                        tagId: "22222222-2222-2222-2222-222222222222",
                        tagTypeId: "44444444-4444-4444-4444-444444444444",
                    },
                ],
            ) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
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
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn media_succeeds() {
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
                &false,
            )
        })
        .returning(|_, _, _, _| {
            Ok(vec![
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
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
                "createdAt": "2022-06-01T12:34:56",
                "updatedAt": "2022-06-01T00:05:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "createdAt": "2022-06-01T12:34:58",
                "updatedAt": "2022-06-01T00:05:02",
            },
        ],
    }));
}

#[tokio::test]
async fn media_with_tags_succeeds() {
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
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: {
                        let mut tags = BTreeMap::new();
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                slug: "character".to_string(),
                                name: "キャラクター".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                    name: "赤座あかり".to_string(),
                                    kana: "あかざあかり".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                    parent: Some(Box::new(tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                                tags::Tag {
                                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: Default::default(),
                                    parent: Some(Box::new(tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                },
                            ],
                        );
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                slug: "work".to_string(),
                                name: "作品".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: vec![
                                        tags::Tag {
                                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                            name: "赤座あかり".to_string(),
                                            kana: "あかざあかり".to_string(),
                                            aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        },
                                        tags::Tag {
                                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: {
                        let mut tags = BTreeMap::new();
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                slug: "character".to_string(),
                                name: "キャラクター".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                    name: "赤座あかり".to_string(),
                                    kana: "あかざあかり".to_string(),
                                    aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                    parent: Some(Box::new(tags::Tag {
                                        id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                        name: "ゆるゆり".to_string(),
                                        kana: "ゆるゆり".to_string(),
                                        aliases: Default::default(),
                                        parent: None,
                                        children: Vec::new(),
                                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                    })),
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags.insert(
                            tag_types::TagType {
                                id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                                slug: "work".to_string(),
                                name: "作品".to_string(),
                            },
                            vec![
                                tags::Tag {
                                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                                    name: "ゆるゆり".to_string(),
                                    kana: "ゆるゆり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: vec![
                                        tags::Tag {
                                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                            name: "赤座あかり".to_string(),
                                            kana: "あかざあかり".to_string(),
                                            aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                        },
                                        tags::Tag {
                                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                            name: "歳納京子".to_string(),
                                            kana: "としのうきょうこ".to_string(),
                                            aliases: Default::default(),
                                            parent: None,
                                            children: Vec::new(),
                                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                        },
                                    ],
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
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
                                "createdAt": "2022-06-01T00:00:00",
                                "updatedAt": "2022-06-01T00:01:00",
                            },
                            "children": [],
                            "createdAt": "2022-06-01T00:00:00",
                            "updatedAt": "2022-06-01T00:01:00",
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
                                "createdAt": "2022-06-01T00:00:00",
                                "updatedAt": "2022-06-01T00:01:00",
                            },
                            "children": [],
                            "createdAt": "2022-06-01T00:02:00",
                            "updatedAt": "2022-06-01T00:03:00",
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
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                {
                                    "id": "55555555-5555-5555-5555-555555555555",
                                    "name": "歳納京子",
                                    "kana": "としのうきょうこ",
                                    "aliases": [],
                                    "children": [],
                                    "createdAt": "2022-06-01T00:02:00",
                                    "updatedAt": "2022-06-01T00:03:00",
                                },
                            ],
                            "createdAt": "2022-06-01T00:00:00",
                            "updatedAt": "2022-06-01T00:01:00",
                        },
                        "type": {
                            "id": "66666666-6666-6666-6666-666666666666",
                            "slug": "work",
                            "name": "作品",
                        },
                    },
                ],
                "createdAt": "2022-06-01T12:34:56",
                "updatedAt": "2022-06-01T00:05:00",
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
                                "createdAt": "2022-06-01T00:00:00",
                                "updatedAt": "2022-06-01T00:01:00",
                            },
                            "children": [],
                            "createdAt": "2022-06-01T00:00:00",
                            "updatedAt": "2022-06-01T00:01:00",
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
                                    "createdAt": "2022-06-01T00:00:00",
                                    "updatedAt": "2022-06-01T00:01:00",
                                },
                                {
                                    "id": "55555555-5555-5555-5555-555555555555",
                                    "name": "歳納京子",
                                    "kana": "としのうきょうこ",
                                    "aliases": [],
                                    "children": [],
                                    "createdAt": "2022-06-01T00:02:00",
                                    "updatedAt": "2022-06-01T00:03:00",
                                },
                            ],
                            "createdAt": "2022-06-01T00:00:00",
                            "updatedAt": "2022-06-01T00:01:00",
                        },
                        "type": {
                            "id": "66666666-6666-6666-6666-666666666666",
                            "slug": "work",
                            "name": "作品",
                        },
                    },
                ],
                "createdAt": "2022-06-01T12:34:58",
                "updatedAt": "2022-06-01T00:05:02",
            },
        ],
    }));
}

#[tokio::test]
async fn media_with_replicas_succeeds() {
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
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: vec![
                        replicas::Replica {
                            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                            display_order: Some(1),
                            has_thumbnail: true,
                            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                        },
                        replicas::Replica {
                            id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                            display_order: Some(2),
                            has_thumbnail: true,
                            original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                            mime_type: "image/png".to_string(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
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
            media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                id
                replicas {
                    id
                    displayOrder
                    originalUrl
                    thumbnailUrl
                    mimeType
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
                        "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                        "thumbnailUrl": "https://img.example.com/66666666-6666-6666-6666-666666666666",
                        "mimeType": "image/png",
                        "createdAt": "2022-06-02T00:00:00",
                        "updatedAt": "2022-06-02T00:01:00",
                    },
                    {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "displayOrder": 2,
                        "originalUrl": "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png",
                        "thumbnailUrl": "https://img.example.com/77777777-7777-7777-7777-777777777777",
                        "mimeType": "image/png",
                        "createdAt": "2022-06-03T00:02:00",
                        "updatedAt": "2022-06-03T00:03:00",
                    },
                ],
                "createdAt": "2022-06-01T12:34:56",
                "updatedAt": "2022-06-01T00:05:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "replicas": [],
                "createdAt": "2022-06-01T12:34:58",
                "updatedAt": "2022-06-01T00:05:02",
            },
        ],
    }));
}

#[tokio::test]
async fn media_with_sources_succeeds() {
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
                media::Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: vec![
                        sources::Source {
                            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                            external_service: external_services::ExternalService {
                                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                slug: "twitter".to_string(),
                                name: "Twitter".to_string(),
                            },
                            external_metadata: external_services::ExternalMetadata::Twitter { id: 727620202049900544 },
                            created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                        },
                        sources::Source {
                            id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                            external_service: external_services::ExternalService {
                                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                slug: "pixiv".to_string(),
                                name: "pixiv".to_string(),
                            },
                            external_metadata: external_services::ExternalMetadata::Pixiv { id: 56736941 },
                            created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
                        },
                    ],
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                media::Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
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
                        "createdAt": "2016-05-04T07:05:00",
                        "updatedAt": "2016-05-04T07:05:01",
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
                        "createdAt": "2016-05-06T05:14:00",
                        "updatedAt": "2016-05-06T05:14:01",
                    },
                ],
                "createdAt": "2022-06-01T12:34:56",
                "updatedAt": "2022-06-01T00:05:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "sources": [],
                "createdAt": "2022-06-01T12:34:58",
                "updatedAt": "2022-06-01T00:05:02",
            },
        ],
    }));
}
