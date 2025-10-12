use std::{borrow::Cow, sync::Arc};

use async_graphql::{Schema, EmptySubscription, value};
use domain::entity::external_services::{ExternalService, ExternalServiceId, ExternalServiceKind};
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
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_create_external_service()
        .times(1)
        .withf(|slug, kind, name, base_url, url_pattern| (slug, kind, name, base_url, url_pattern) == (
            "x",
            &ExternalServiceKind::X,
            "X",
            &Some("https://x.com"),
            &Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$"),
         ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                slug: "x".to_string(),
                kind: ExternalServiceKind::X,
                name: "X".to_string(),
                base_url: Some("https://x.com".to_string()),
                url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
            }))
        });

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();

    let mut normalizer = MockNormalizerInterface::new();
    normalizer
        .expect_normalize_str()
        .times(2)
        .withf(|text| text == "x")
        .returning(|_| Cow::from("x"));

    normalizer
        .expect_normalize_str()
        .times(1)
        .withf(|text| text == "X")
        .returning(|_| Cow::from("X"));

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
            createExternalService(
                slug: "x",
                kind: "x",
                name: "X",
                baseUrl: "https://x.com",
                urlPattern: "^https?://(?:twitter\\.com|x\\.com)/(?<creatorId>[^/]+)/status/(?<id>\\d+)(?:[/?#].*)?$",
            ) {
                id
                slug
                kind
                name
                baseUrl
                urlPattern
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "createExternalService": {
            "id": "33333333-3333-3333-3333-333333333333",
            "slug": "x",
            "kind": "x",
            "name": "X",
            "baseUrl": "https://x.com",
            "urlPattern": r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$",
        },
    }));
}
