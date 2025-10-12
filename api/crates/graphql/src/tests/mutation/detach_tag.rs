use std::{collections::BTreeSet, sync::Arc};

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
    let normalizer = MockNormalizerInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_detach_tag_by_id()
        .times(1)
        .withf(|id, depth| (id, depth) == (
            &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            &TagDepth::new(1, 0),
        ))
        .returning(|_, _| {
            Box::pin(ok(Tag {
                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
            }))
        });

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface, MockQueryParserInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        mutation {
            detachTag(id: "33333333-3333-3333-3333-333333333333") {
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
        "detachTag": {
            "id": "33333333-3333-3333-3333-333333333333",
            "name": "赤座あかり",
            "kana": "あかざあかり",
            "aliases": ["アッカリーン"],
            "parent": null,
            "createdAt": "2022-06-01T00:00:00+00:00",
            "updatedAt": "2022-06-01T00:01:00+00:00",
        },
    }));
}
