use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::{
    entity::replicas::{self, ReplicaId},
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
        .expect_get_replica_by_original_url()
        .times(1)
        .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
        .returning(|_| {
            Ok(replicas::Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: Some(1),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
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
                originalUrl
                thumbnailUrl
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
            "originalUrl": "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
            "thumbnailUrl": "https://img.example.com/66666666-6666-6666-6666-666666666666",
            "mimeType": "image/png",
            "createdAt": "2022-06-02T00:00:00",
            "updatedAt": "2022-06-02T00:01:00",
        },
    }));
}
