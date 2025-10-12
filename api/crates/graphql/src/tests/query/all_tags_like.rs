use std::{borrow::Cow, collections::BTreeSet, sync::Arc};

use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::entity::tags::{AliasSet, Tag, TagDepth, TagId};
use futures::future::ok;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::query::Query;

use super::mocks::{
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
    query::MockQueryParserInterface,
};

#[tokio::test]
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tags_by_name_or_alias_like()
        .times(1)
        .withf(|query, depth| query.clone_box().eq(["ゆ".to_string(), "り".to_string()].into_iter()) && depth == &TagDepth::new(2, 2))
        .returning(|_, _| {
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
            ]))
        });

    let mut normalizer = MockNormalizerInterface::new();
    normalizer
        .expect_normalize_str()
        .times(1)
        .withf(|text| text == "ゆ り")
        .returning(|_| Cow::from("ゆ り"));

    let mut query_parser = MockQueryParserInterface::new();
    query_parser
        .expect_parse()
        .times(1)
        .withf(|query| query == "ゆ り")
        .returning(|_| vec!["ゆ", "り"]);

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface, MockQueryParserInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .data(query_parser)
        .finish();

    let req = indoc! {r#"
        query {
            allTagsLike(nameOrAliasLike: "ゆ り") {
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
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTagsLike": [
            {
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
            {
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
        ],
    }));
}
