use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{self, ExternalServiceId},
        sources::{self, SourceId},
    },
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
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_source_by_external_metadata()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &external_services::ExternalMetadata::Twitter { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            )
        })
        .returning(|_, _| {
            Box::pin(ok(Some(sources::Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "twitter".to_string(),
                    kind: "twitter".to_string(),
                    name: "Twitter".to_string(),
                    base_url: Some("https://twitter.com".to_string()),
                },
                external_metadata: external_services::ExternalMetadata::Twitter { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            })))
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .finish();

    let req = indoc! {r#"
        query {
            source(
                externalServiceId: "33333333-3333-3333-3333-333333333333",
                externalMetadata: {
                    twitter: {
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
                }
                externalMetadata
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "source": {
            "id": "11111111-1111-1111-1111-111111111111",
            "externalService": {
                "id": "33333333-3333-3333-3333-333333333333",
                "slug": "twitter",
                "kind": "twitter",
                "name": "Twitter",
                "baseUrl": "https://twitter.com",
            },
            "externalMetadata": {
                "twitter": {
                    "id": "727620202049900544",
                    "creatorId": "_namori_",
                },
            },
            "createdAt": "2016-05-04T07:05:00+00:00",
            "updatedAt": "2016-05-04T07:05:01+00:00",
        },
    }));
}

#[tokio::test]
async fn not_found() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_source_by_external_metadata()
        .times(1)
        .withf(|external_service_id, external_metadata| {
            (external_service_id, external_metadata) == (
                &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &external_services::ExternalMetadata::Twitter { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
            )
        })
        .returning(|_, _| Box::pin(ok(None)));

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .finish();

    let req = indoc! {r#"
        query {
            source(
                externalServiceId: "33333333-3333-3333-3333-333333333333",
                externalMetadata: {
                    twitter: {
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
                }
                externalMetadata
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "source": null,
    }));
}
