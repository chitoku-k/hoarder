use async_graphql::{value, EmptySubscription, Schema};
use chrono::{TimeZone, Utc};
use domain::entity::{
    external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
    sources::{Source, SourceId},
};
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
        .expect_update_source_by_id()
        .times(1)
        .withf(|id, external_service_id, external_metadata| (id, external_service_id, external_metadata) == (
            &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
            &Some(ExternalMetadata::Pixiv { id: 56736941 }),
        ))
        .returning(|_, _, _| {
            Box::pin(ok(Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: ExternalServiceKind::Pixiv,
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
                external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
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
            updateSource(
                id: "11111111-1111-1111-1111-111111111111",
                externalServiceId: "11111111-1111-1111-1111-111111111111",
                externalMetadata: {
                    pixiv: {
                        id: "56736941",
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
        "updateSource": {
            "id": "11111111-1111-1111-1111-111111111111",
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
            "createdAt": "2016-05-04T07:05:00+00:00",
            "updatedAt": "2016-05-04T07:05:01+00:00",
        },
    }));
}
