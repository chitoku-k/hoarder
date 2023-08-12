use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
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
use chrono::NaiveDate;
use graphql::query::Query;
use indoc::indoc;
use pretty_assertions::assert_eq;
use thumbnails::ThumbnailURLFactory;
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
                &external_services::ExternalMetadata::Twitter { id: 727620202049900544 },
            )
        })
        .returning(|_, _| {
            Ok(sources::Source {
                id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                external_service: external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
                external_metadata: external_services::ExternalMetadata::Twitter { id: 727620202049900544 },
                created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
            })
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(ThumbnailURLFactory::new("https://img.example.com/".to_string()))
        .finish();
    let req = indoc! {r#"
        query {
            source(
                externalServiceId: "33333333-3333-3333-3333-333333333333",
                externalMetadata: {
                    twitter: {
                        id: 727620202049900544,
                    },
                },
            ) {
                id
                externalService {
                    id
                    slug
                    name
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
                "name": "Twitter",
            },
            "externalMetadata": {
                "twitter": {
                    "id": "727620202049900544",
                },
            },
            "createdAt": "2016-05-04T07:05:00",
            "updatedAt": "2016-05-04T07:05:01",
        },
    }));
}