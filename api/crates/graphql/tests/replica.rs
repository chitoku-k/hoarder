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
        .expect_get_replica_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
        .returning(|_| {
            Ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: Some(Thumbnail {
                    id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    size: Size::new(240, 240),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                }),
                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            })
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(ThumbnailURLFactory::new("https://img.example.com/".to_string()))
        .finish();
    let req = indoc! {r#"
        query {
            replica(originalUrl: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png") {
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
                originalUrl
                mimeType
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
            "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
            "mimeType": "image/png",
            "createdAt": "2022-06-02T00:00:00+00:00",
            "updatedAt": "2022-06-02T00:01:00+00:00",
        },
    }));
}
