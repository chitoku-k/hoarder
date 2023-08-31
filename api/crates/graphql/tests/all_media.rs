use std::collections::BTreeMap;

use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        sources::SourceId,
        tag_types::TagTypeId,
        tags::TagId,
    },
    repository::{Direction, Order},
    service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
};
use graphql::{query::Query, tags::TagTagTypeInput};
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::{uuid, Uuid};

// Concrete type is required both in implementation and expectation.
type IntoIterMap<T, U> = std::iter::Map<std::vec::IntoIter<T>, fn(T) -> U>;

#[tokio::test]
async fn by_source_ids_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_source_ids::<IntoIterMap<Uuid, SourceId>>()
        .times(1)
        .withf(|source_ids, tag_depth, replicas, sources, cursor, order, direction, limit| {
            source_ids.clone().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ]) &&
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, sourceIds: ["11111111-1111-1111-1111-111111111111", "33333333-3333-3333-3333-333333333333"]) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
                        createdAt
                        updatedAt
                    }
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allMedia": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57+00:00",
                        "updatedAt": "2022-06-01T00:05:01+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn by_tag_ids_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media_by_tag_ids::<IntoIterMap<TagTagTypeInput, (TagId, TagTypeId)>>()
        .times(1)
        .withf(|tag_ids, tag_depth, replicas, sources, cursor, order, direction, limit| {
            tag_ids.clone().eq([
                (
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(
                first: 3,
                tagIds: [
                    {
                        tagId: "22222222-2222-2222-2222-222222222222",
                        tagTypeId: "44444444-4444-4444-4444-444444444444",
                    },
                ],
            ) {
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node {
                        id
                        createdAt
                        updatedAt
                    }
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allMedia": {
            "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57+00:00",
                        "updatedAt": "2022-06-01T00:05:01+00:00",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58+00:00",
                        "updatedAt": "2022-06-01T00:05:02+00:00",
                    },
                },
            ],
        },
    }));
}
