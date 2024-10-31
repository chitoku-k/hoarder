use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::media::{Medium, MediumId},
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
use ordermap::OrderMap;
use pretty_assertions::assert_eq;
use uuid::{uuid, Uuid};

// Concrete type is required both in implementation and expectation.
type IntoIterMap<T, U> = std::iter::Map<std::vec::IntoIter<T>, fn(T) -> U>;

#[tokio::test]
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_ids::<IntoIterMap<Uuid, MediumId>>()
        .times(1)
        .withf(|ids, tag_depth, replicas, sources| {
            ids.clone().eq([
                MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ]) &&
            (tag_depth, replicas, sources) == (
                &None,
                &false,
                &false,
            )
        })
        .returning(|_, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
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
            media(ids: ["77777777-7777-7777-7777-777777777777" "99999999-9999-9999-9999-999999999999"]) {
                id
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "media": [
            {
                "id": "77777777-7777-7777-7777-777777777777",
                "createdAt": "2022-06-01T12:34:56+00:00",
                "updatedAt": "2022-06-01T00:05:00+00:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "createdAt": "2022-06-01T12:34:58+00:00",
                "updatedAt": "2022-06-01T00:05:02+00:00",
            },
        ],
    }));
}
