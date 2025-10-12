use std::{borrow::Cow, sync::Arc};

use async_graphql::{value, EmptySubscription, Schema};
use domain::entity::tag_types::{TagType, TagTypeId};
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
        .expect_create_tag_type()
        .times(1)
        .withf(|slug, name, kana| (slug, name, kana) == (
            "character",
            "キャラクター",
            "キャラクター",
        ))
        .returning(|_, _, _| {
            Box::pin(ok(TagType {
                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                slug: "character".to_string(),
                name: "キャラクター".to_string(),
                kana: "キャラクター".to_string(),
            }))
        });

    let mut normalizer = MockNormalizerInterface::new();
    normalizer
        .expect_normalize_str()
        .times(1)
        .withf(|text| text == "character")
        .returning(|_| Cow::from("character"));

    normalizer
        .expect_normalize_str()
        .times(2)
        .withf(|text| text == "キャラクター")
        .returning(|_| Cow::from("キャラクター"));

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
            createTagType(
                slug: "character",
                name: "キャラクター",
                kana: "キャラクター",
            ) {
                id
                slug
                name
                kana
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "createTagType": {
            "id": "44444444-4444-4444-4444-444444444444",
            "slug": "character",
            "name": "キャラクター",
            "kana": "キャラクター",
        },
    }));
}
