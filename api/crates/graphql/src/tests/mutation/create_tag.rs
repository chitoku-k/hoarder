use std::{borrow::Cow, collections::BTreeSet};

use async_graphql::{value, EmptySubscription, Schema};
use chrono::{TimeZone, Utc};
use domain::entity::tags::{AliasSet, Tag, TagDepth, TagId};
use futures::future::ok;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::{mutation::Mutation, query::Query};

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
        .expect_create_tag()
        .times(1)
        .withf(|name, kana, aliases, parent_id, depth| {
            aliases.clone_box().eq(["アッカリーン".to_string()]) &&
            (name, kana, parent_id, depth) == (
                "赤座あかり",
                "あかざあかり",
                &Some(TagId::from(uuid!("22222222-2222-2222-2222-222222222222"))),
                &TagDepth::new(1, 0),
            )
        })
        .returning(|_, _, _, _, _| {
            Box::pin(ok(Tag {
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
            }))
        });

    let mut normalizer = MockNormalizerInterface::new();
    normalizer
        .expect_normalize_str()
        .times(1)
        .withf(|text| text == "赤座あかり")
        .returning(|_| Cow::from("赤座あかり"));

    normalizer
        .expect_normalize_str()
        .times(1)
        .withf(|text| text == "あかざあかり")
        .returning(|_| Cow::from("あかざあかり"));

    normalizer
        .expect_normalize_str()
        .times(1)
        .withf(|text| text == "アッカリーン")
        .returning(|_| Cow::from("アッカリーン"));

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface, MockQueryParserInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(normalizer)
        .finish();

    let req = indoc! {r#"
        mutation {
            createTag(
                name: "赤座あかり",
                kana: "あかざあかり",
                aliases: ["アッカリーン"],
                parentId: "22222222-2222-2222-2222-222222222222",
            ) {
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
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "createTag": {
            "id": "33333333-3333-3333-3333-333333333333",
            "name": "赤座あかり",
            "kana": "あかざあかり",
            "aliases": ["アッカリーン"],
            "parent": {
                "id": "22222222-2222-2222-2222-222222222222",
                "name": "ゆるゆり",
                "kana": "ゆるゆり",
                "aliases": [],
                "createdAt": "2022-06-01T00:00:00+00:00",
                "updatedAt": "2022-06-01T00:01:00+00:00",
            },
            "createdAt": "2022-06-01T00:00:00+00:00",
            "updatedAt": "2022-06-01T00:01:00+00:00",
        },
    }));
}
