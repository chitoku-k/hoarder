use std::{borrow::Cow, sync::Arc};

use async_graphql::{Schema, EmptySubscription, value};
use domain::entity::external_services::{ExternalService, ExternalServiceId};
use futures::future::ok;
use graphql::{mutation::Mutation, query::Query};
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::uuid;

mod mocks;
use mocks::{
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
};

#[tokio::test]
async fn succeeds() {
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_update_external_service_by_id()
        .times(1)
        .withf(|id, slug, name, base_url, url_pattern| (id, slug, name, base_url, url_pattern) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &Some("PIXIV"),
            &Some("PIXIV"),
            &Some(Some("https://example.com")),
            &Some(Some(r"^https://example\.com")),
         ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                kind: "PIXIV".to_string(),
                name: "PIXIV".to_string(),
                base_url: Some("https://example.com".to_string()),
                url_pattern: Some(r"^https://example\.com".to_string()),
            }))
        });

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();

    let mut normalizer = MockNormalizerInterface::new();
    normalizer
        .expect_normalize_str()
        .times(2)
        .withf(|text| text == "PIXIV")
        .returning(|_| Cow::from("PIXIV"));

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        mutation {
            updateExternalService(
                id: "11111111-1111-1111-1111-111111111111",
                slug: "PIXIV",
                name: "PIXIV",
                baseUrl: "https://example.com",
                urlPattern: "^https://example\\.com",
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
        "updateExternalService": {
            "id": "11111111-1111-1111-1111-111111111111",
            "slug": "pixiv",
            "kind": "PIXIV",
            "name": "PIXIV",
            "baseUrl": "https://example.com",
            "urlPattern": r"^https://example\.com",
        },
    }));
}

#[tokio::test]
async fn succeeds_empty() {
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_update_external_service_by_id()
        .times(1)
        .withf(|id, slug, name, base_url, url_pattern| (id, slug, name, base_url, url_pattern) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &None,
            &None,
            &Some(None),
            &Some(None),
         ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                kind: "pixiv".to_string(),
                name: "pixiv".to_string(),
                base_url: None,
                url_pattern: None,
            }))
        });

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        mutation {
            updateExternalService(
                id: "11111111-1111-1111-1111-111111111111",
                baseUrl: "",
                urlPattern: "",
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
        "updateExternalService": {
            "id": "11111111-1111-1111-1111-111111111111",
            "slug": "pixiv",
            "kind": "pixiv",
            "name": "pixiv",
            "baseUrl": null,
            "urlPattern": null,
        },
    }));
}

#[tokio::test]
async fn succeeds_none() {
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_update_external_service_by_id()
        .times(1)
        .withf(|id, slug, name, base_url, url_pattern| (id, slug, name, base_url, url_pattern) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &None,
            &None,
            &None,
            &None,
         ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                kind: "pixiv".to_string(),
                name: "pixiv".to_string(),
                base_url: Some("https://www.pixiv.net".to_string()),
                url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
            }))
        });

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        mutation {
            updateExternalService(id: "11111111-1111-1111-1111-111111111111") {
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
        "updateExternalService": {
            "id": "11111111-1111-1111-1111-111111111111",
            "slug": "pixiv",
            "kind": "pixiv",
            "name": "pixiv",
            "baseUrl": "https://www.pixiv.net",
            "urlPattern": r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
        },
    }));
}
