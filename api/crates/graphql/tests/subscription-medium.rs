use std::sync::Arc;

use application::service::media::MediaURLFactoryInterface;
use async_graphql::{Schema, EmptyMutation, value};
use chrono::{TimeZone, Utc};
use domain::entity::{media::{Medium, MediumId}, replicas::{Replica, ReplicaId, ReplicaStatus}};
use futures::{future::ok, stream, StreamExt};
use graphql::{query::Query, subscription::Subscription};
use indoc::indoc;
use ordermap::OrderMap;
use pretty_assertions::assert_eq;
use uuid::uuid;

mod mocks;
use mocks::{
    application::service::media::MockMediaURLFactoryInterface,
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
    let tags_service = MockTagsServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_watch_medium_by_id()
        .times(1)
        .withf(|id, tag_depth, replica, sources| (id, tag_depth, replica, sources) == (
            &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            &None,
            &true,
            &false,
        ))
        .returning(|_, _, _, _| {
            Box::pin(ok(stream::iter([
                Ok(Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                }),
                Ok(Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: vec![
                        Replica {
                            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                            display_order: 1,
                            thumbnail: None,
                            original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                            mime_type: None,
                            size: None,
                            status: ReplicaStatus::Processing,
                            created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                }),
            ]).boxed()))
        });

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Some("https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string()));

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let subscription = Subscription::<MockMediaServiceInterface>::new();
    let schema = Schema::build(query, EmptyMutation, subscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .finish();

    let req = indoc! {r#"
        subscription {
            medium(id: "77777777-7777-7777-7777-777777777777") {
                id
                replicas {
                    id
                    displayOrder
                    thumbnail {
                        id
                        url
                        width
                        height
                        createdAt
                        updatedAt
                    }
                    url
                    originalUrl
                    mimeType
                    width
                    height
                    status {
                        phase
                    }
                    createdAt
                    updatedAt
                }
                createdAt
                updatedAt
            }
        }
    "#};

    let actual: Vec<_> = schema.execute_stream(req).collect().await;

    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0].data, value!({
        "medium": {
            "id": "77777777-7777-7777-7777-777777777777",
            "replicas": [],
            "createdAt": "2022-06-01T12:34:56+00:00",
            "updatedAt": "2022-06-01T00:05:00+00:00",
        },
    }));
    assert_eq!(actual[1].data, value!({
        "medium": {
            "id": "77777777-7777-7777-7777-777777777777",
            "replicas": [
                {
                    "id": "66666666-6666-6666-6666-666666666666",
                    "displayOrder": 1,
                    "thumbnail": null,
                    "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
                    "originalUrl": "file:///77777777-7777-7777-7777-777777777777.png",
                    "mimeType": null,
                    "width": null,
                    "height": null,
                    "status": {
                        "phase": "PROCESSING",
                    },
                    "createdAt": "2022-06-02T00:00:00+00:00",
                    "updatedAt": "2022-06-02T00:01:00+00:00",
                },
            ],
            "createdAt": "2022-06-01T12:34:56+00:00",
            "updatedAt": "2022-06-01T00:05:00+00:00",
        },
    }));
}
