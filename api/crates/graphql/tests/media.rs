use std::collections::BTreeMap;

use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::{
    entity::media::{Medium, MediumId},
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
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
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
                "createdAt": "2022-06-01T12:34:56",
                "updatedAt": "2022-06-01T00:05:00",
            },
            {
                "id": "99999999-9999-9999-9999-999999999999",
                "createdAt": "2022-06-01T12:34:58",
                "updatedAt": "2022-06-01T00:05:02",
            },
        ],
    }));
}
