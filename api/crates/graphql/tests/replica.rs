use std::sync::Arc;

use application::service::{
    media::{MediaURLFactoryInterface, MockMediaURLFactoryInterface},
    thumbnails::{MockThumbnailURLFactoryInterface, ThumbnailURLFactoryInterface},
};
use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::replicas::{Replica, ReplicaId, Size, Thumbnail, ThumbnailId},
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
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_replica_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: Some(Thumbnail {
                    id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    size: Size::new(240, 240),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                }),
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                size: Size::new(720, 720),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let tags_service = MockTagsServiceInterface::new();

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Some("https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string()));

    let mut thumbnail_url_factory = MockThumbnailURLFactoryInterface::new();
    thumbnail_url_factory
        .expect_get()
        .times(1)
        .withf(|thumbnail_id| thumbnail_id == &ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")))
        .returning(|_| "https://img.example.com/88888888-8888-8888-8888-888888888888".to_string());

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .data::<Arc<dyn ThumbnailURLFactoryInterface>>(Arc::new(thumbnail_url_factory))
        .finish();

    let req = indoc! {r#"
        query {
            replica(originalUrl: "file:///77777777-7777-7777-7777-777777777777.png") {
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
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "replica": {
            "id": "66666666-6666-6666-6666-666666666666",
            "displayOrder": 1,
            "thumbnail": {
                "id": "88888888-8888-8888-8888-888888888888",
                "url": "https://img.example.com/88888888-8888-8888-8888-888888888888",
                "width": 240,
                "height": 240,
                "createdAt": "2022-06-02T00:02:00+00:00",
                "updatedAt": "2022-06-02T00:03:00+00:00",
            },
            "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
            "originalUrl": "file:///77777777-7777-7777-7777-777777777777.png",
            "mimeType": "image/png",
            "width": 720,
            "height": 720,
            "createdAt": "2022-06-02T00:00:00+00:00",
            "updatedAt": "2022-06-02T00:01:00+00:00",
        },
    }));
}
