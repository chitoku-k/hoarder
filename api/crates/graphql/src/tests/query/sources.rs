use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::entity::{
    external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
    sources::{Source, SourceId},
};
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
};

#[tokio::test]
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_sources_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        ]))
        .returning(|_| {
            Box::pin(ok(vec![
                Source {
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
                },
                Source {
                    id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        slug: "pixiv".to_string(),
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                    created_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2016, 5, 6, 5, 14, 1).unwrap(),
                },
            ]))
        });

    let tags_service = MockTagsServiceInterface::new();
    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .finish();

    let req = indoc! {r#"
        query {
            sources(ids: ["11111111-1111-1111-1111-111111111111", "22222222-2222-2222-2222-222222222222"]) {
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
        "sources": [
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
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "externalService": {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "slug": "pixiv",
                    "kind": "pixiv",
                    "name": "pixiv",
                    "baseUrl": "https://www.pixiv.net",
                    "urlPattern": r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
                },
                "externalMetadata": {
                    "pixiv": {
                        "id": "56736941",
                    },
                },
                "url": "https://www.pixiv.net/artworks/56736941",
                "createdAt": "2016-05-06T05:14:00+00:00",
                "updatedAt": "2016-05-06T05:14:01+00:00",
            },
        ],
    }));
}
