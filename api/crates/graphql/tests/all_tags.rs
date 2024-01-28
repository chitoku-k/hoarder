use std::collections::BTreeSet;

use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    repository::{Direction, Order},
    service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
};
use futures::future::ok;
use graphql::query::Query;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::uuid;

#[tokio::test]
async fn root_first_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tags()
        .times(1)
        .withf(|depth, root, cursor, order, direction, limit| {
            (depth, root, cursor, order, direction, limit) == (
                &TagDepth::new(2, 2),
                &true,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Box::pin(ok(vec![
                Tag {
                    id: TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    name: "アークナイツ".to_string(),
                    kana: "アークナイツ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    name: "原神".to_string(),
                    kana: "げんしん".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    name: "ブルーアーカイブ".to_string(),
                    kana: "ブルーアーカイブ".to_string(),
                    aliases: AliasSet::default(),
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
            ]))
        });

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allTags(root: true, first: 3) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTags": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": true,
            },
            "edges": [
                {
                    "node": {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "name": "アークナイツ",
                        "kana": "アークナイツ",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:04:00+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "name": "原神",
                        "kana": "げんしん",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:03:00+00:00",
                        "updatedAt": "2022-06-01T00:04:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "66666666-6666-6666-6666-666666666666",
                        "name": "ブルーアーカイブ",
                        "kana": "ブルーアーカイブ",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:02:00+00:00",
                        "updatedAt": "2022-06-01T00:03:00+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn root_last_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tags()
        .times(1)
        .withf(|depth, root, cursor, order, direction, limit| {
            (depth, root, cursor, order, direction, limit) == (
                &TagDepth::new(2, 2),
                &true,
                &None,
                &Order::Descending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Box::pin(ok(vec![
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
                Tag {
                    id: TagId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    name: "ブルーアーカイブ".to_string(),
                    kana: "ブルーアーカイブ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    name: "原神".to_string(),
                    kana: "げんしん".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    name: "アークナイツ".to_string(),
                    kana: "アークナイツ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
            ]))
        });

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allTags(root: true, last: 3) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTags": {
            "pageInfo": {
                "hasPreviousPage": true,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "name": "原神",
                        "kana": "げんしん",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:03:00+00:00",
                        "updatedAt": "2022-06-01T00:04:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "66666666-6666-6666-6666-666666666666",
                        "name": "ブルーアーカイブ",
                        "kana": "ブルーアーカイブ",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:02:00+00:00",
                        "updatedAt": "2022-06-01T00:03:00+00:00",
                    },
                },
                {
                    "node": {
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
                },
            ],
        },
    }));
}

#[tokio::test]
async fn root_after_first_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tags()
        .times(1)
        .withf(|depth, root, cursor, order, direction, limit| {
            (depth, root, cursor, order, direction, limit) == (
                &TagDepth::new(2, 2),
                &true,
                &Some(("ブルーアーカイブ".to_string(), TagId::from(uuid!("66666666-6666-6666-6666-666666666666")))),
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Box::pin(ok(vec![
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
            ]))
        });

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allTags(root: true, first: 3, after: "44OW44Or44O844Ki44O844Kr44Kk44OWADY2NjY2NjY2LTY2NjYtNjY2Ni02NjY2LTY2NjY2NjY2NjY2Ng==") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTags": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
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
                },
            ],
        },
    }));
}

#[tokio::test]
async fn root_after_last_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tags()
        .times(1)
        .withf(|depth, root, cursor, order, direction, limit| {
            (depth, root, cursor, order, direction, limit) == (
                &TagDepth::new(2, 2),
                &true,
                &Some(("げんしん".to_string(), TagId::from(uuid!("44444444-4444-4444-4444-444444444444")))),
                &Order::Descending,
                &Direction::Backward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Box::pin(ok(vec![
                Tag {
                    id: TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    name: "アークナイツ".to_string(),
                    kana: "アークナイツ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
            ]))
        });

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allTags(root: true, last: 3, after: "44GS44KT44GX44KTADQ0NDQ0NDQ0LTQ0NDQtNDQ0NC00NDQ0LTQ0NDQ0NDQ0NDQ0NA==") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTags": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "name": "アークナイツ",
                        "kana": "アークナイツ",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:04:00+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn root_before_first_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tags()
        .times(1)
        .withf(|depth, root, cursor, order, direction, limit| {
            (depth, root, cursor, order, direction, limit) == (
                &TagDepth::new(2, 2),
                &true,
                &Some(("ブルーアーカイブ".to_string(), TagId::from(uuid!("66666666-6666-6666-6666-666666666666")))),
                &Order::Ascending,
                &Direction::Backward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Box::pin(ok(vec![
                Tag {
                    id: TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    name: "アークナイツ".to_string(),
                    kana: "アークナイツ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    name: "原神".to_string(),
                    kana: "げんしん".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                },
            ]))
        });

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allTags(root: true, first: 3, before: "44OW44Or44O844Ki44O844Kr44Kk44OWADY2NjY2NjY2LTY2NjYtNjY2Ni02NjY2LTY2NjY2NjY2NjY2Ng==") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTags": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "name": "アークナイツ",
                        "kana": "アークナイツ",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:04:00+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "name": "原神",
                        "kana": "げんしん",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:03:00+00:00",
                        "updatedAt": "2022-06-01T00:04:00+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn root_before_last_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tags()
        .times(1)
        .withf(|depth, root, cursor, order, direction, limit| {
            (depth, root, cursor, order, direction, limit) == (
                &TagDepth::new(2, 2),
                &true,
                &Some(("ブルーアーカイブ".to_string(), TagId::from(uuid!("66666666-6666-6666-6666-666666666666")))),
                &Order::Descending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Box::pin(ok(vec![
                Tag {
                    id: TagId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    name: "原神".to_string(),
                    kana: "げんしん".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    name: "アークナイツ".to_string(),
                    kana: "アークナイツ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 4, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
            ]))
        });

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allTags(root: true, last: 3, before: "44OW44Or44O844Ki44O844Kr44Kk44OWADY2NjY2NjY2LTY2NjYtNjY2Ni02NjY2LTY2NjY2NjY2NjY2Ng==") {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
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
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTags": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "name": "アークナイツ",
                        "kana": "アークナイツ",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:04:00+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "44444444-4444-4444-4444-444444444444",
                        "name": "原神",
                        "kana": "げんしん",
                        "aliases": [],
                        "parent": null,
                        "children": [],
                        "createdAt": "2022-06-01T00:03:00+00:00",
                        "updatedAt": "2022-06-01T00:04:00+00:00",
                    },
                },
            ],
        },
    }));
}
