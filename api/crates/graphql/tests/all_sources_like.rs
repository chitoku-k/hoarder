use std::sync::Arc;

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
use normalizer::MockNormalizerInterface;
use pretty_assertions::assert_eq;
use uuid::uuid;

#[tokio::test]
async fn id_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_sources_by_external_metadata_like_id()
        .times(1)
        .withf(|id| id == "727620202049900544")
        .returning(|_| {
            Box::pin(ok(vec![
                sources::Source {
                    id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    external_service: external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: external_services::ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
                },
            ]))
        });

    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        query {
            allSourcesLike(
                externalMetadataLike: {
                    id: "727620202049900544",
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
        "allSourcesLike": [
            {
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
        ],
    }));
}

#[tokio::test]
async fn url_succeeds() {
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_get_external_services_by_url()
        .times(1)
        .withf(|url| url == "https://x.com/_namori_/status/727620202049900544")
        .returning(|_| {
            Box::pin(ok(vec![
                (
                    external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_services::ExternalMetadata::X {
                        id: 727620202049900544,
                        creator_id: Some("_namori_".to_string()),
                    },
                ),
                (
                    external_services::ExternalService {
                        id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                        slug: "website".to_string(),
                        kind: "website".to_string(),
                        name: "Website".to_string(),
                        base_url: None,
                        url_pattern: None,
                    },
                    external_services::ExternalMetadata::Website {
                        url: "https://x.com/_namori_/status/727620202049900544".to_string(),
                    },
                ),
            ]))
        });

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_source_by_external_metadata()
        .times(1)
        .withf(|external_service_id, external_metadata| (external_service_id, external_metadata) == (
            &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            &external_services::ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
        ))
        .returning(|_, _| {
            Box::pin(ok(Some(sources::Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "x".to_string(),
                    kind: "x".to_string(),
                    name: "X".to_string(),
                    base_url: Some("https://x.com".to_string()),
                    url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                },
                external_metadata: external_services::ExternalMetadata::X { id: 727620202049900544, creator_id: Some("_namori_".to_string()) },
                created_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2016, 5, 4, 7, 5, 1).unwrap(),
            })))
        });

    media_service
        .expect_get_source_by_external_metadata()
        .times(1)
        .withf(|external_service_id, external_metadata| (external_service_id, external_metadata) == (
            &ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            &external_services::ExternalMetadata::Website { url: "https://x.com/_namori_/status/727620202049900544".to_string() },
        ))
        .returning(|_, _| Box::pin(ok(None)));

    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        query {
            allSourcesLike(
                externalMetadataLike: {
                    url: "https://x.com/_namori_/status/727620202049900544",
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
        "allSourcesLike": [
            {
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
        ],
    }));
}
