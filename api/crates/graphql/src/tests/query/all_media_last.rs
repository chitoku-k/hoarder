use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use chrono::{TimeZone, Utc};
use domain::{
    entity::media::{Medium, MediumId},
    repository::{Direction, Order},
};
use futures::future::ok;
use indoc::indoc;
use ordermap::OrderMap;
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
async fn asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Order::Descending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 59).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 3).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
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
            allMedia(last: 3, order: ASC) {
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
                "hasPreviousPage": true,
                "hasNextPage": false,
            },
            "edges": [
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
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59+00:00",
                        "updatedAt": "2022-06-01T00:05:03+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
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
        .returning(|_, _, _, _, _, _, _| {
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
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 59).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 3).unwrap(),
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
            allMedia(last: 3, order: DESC) {
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
                "hasPreviousPage": true,
                "hasNextPage": false,
            },
            "edges": [
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58+00:00",
                        "updatedAt": "2022-06-01T00:05:02+00:00",
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
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn after_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &false,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &Order::Descending,
                &Direction::Backward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 59).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 3).unwrap(),
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
            allMedia(last: 3, after: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDArMDA6MDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
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
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58+00:00",
                        "updatedAt": "2022-06-01T00:05:02+00:00",
                    },
                },
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59+00:00",
                        "updatedAt": "2022-06-01T00:05:03+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn after_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &false,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &Order::Ascending,
                &Direction::Backward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
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
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
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
            allMedia(last: 3, order: DESC, after: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDArMDA6MDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
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
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57+00:00",
                        "updatedAt": "2022-06-01T00:05:01+00:00",
                    },
                },
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56+00:00",
                        "updatedAt": "2022-06-01T00:05:00+00:00",
                    },
                },
            ],
        },
    }));
}

#[tokio::test]
async fn before_asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &false,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &Order::Descending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 1).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
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
            allMedia(last: 3, before: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDArMDA6MDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
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
async fn before_desc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, cursor, order, direction, limit| {
            (tag_depth, replicas, sources, cursor, order, direction, limit) == (
                &None,
                &false,
                &false,
                &Some((Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 57).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &Order::Ascending,
                &Direction::Forward,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Box::pin(ok(vec![
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 58).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 2).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: OrderMap::new(),
                    replicas: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 59).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 3).unwrap(),
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
            allMedia(last: 3, order: DESC, before: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDArMDA6MDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
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
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59+00:00",
                        "updatedAt": "2022-06-01T00:05:03+00:00",
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
