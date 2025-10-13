use async_graphql::{value, EmptySubscription, Schema};
use chrono::{TimeZone, Utc};
use domain::entity::{external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind}, sources::{Source, SourceId}};
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
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_create_source()
        .times(1)
        .withf(|external_service_id, external_metadata| (external_service_id, external_metadata) == (
            &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            &ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
        ))
        .returning(|_, _| {
            Box::pin(ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "x".to_string(),
                    kind: ExternalServiceKind::X,
                    name: "X".to_string(),
                    base_url: Some("https://x.com".to_string()),
                    url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                },
                external_metadata: ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            }))
        });

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
            createSource(
                externalServiceId: "33333333-3333-3333-3333-333333333333",
                externalMetadata: {
                    x: {
                        id: "727620202049900544",
                        creatorId: "_namori_",
                    },
                },
            ) {
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
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "createSource": {
            "id": "11111111-1111-1111-1111-111111111111",
            "externalService": {
                "id": "33333333-3333-3333-3333-333333333333",
                "slug": "x",
                "kind": "x",
                "name": "X",
                "baseUrl": "https://x.com",
                "urlPattern": r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$",
            },
            "externalMetadata": {
                "x": {
                    "id": "727620202049900544",
                    "creatorId": "_namori_",
                },
            },
            "url": "https://x.com/_namori_/status/727620202049900544",
            "createdAt": "2016-05-04T07:05:00+00:00",
            "updatedAt": "2016-05-04T07:05:01+00:00",
        },
    }));
}
