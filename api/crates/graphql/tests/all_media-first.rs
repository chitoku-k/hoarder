use std::collections::BTreeMap;

use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::{
    entity::media::{Medium, MediumId},
    repository::OrderDirection,
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
use uuid::uuid;

#[tokio::test]
async fn asc_succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_media()
        .times(1)
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
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
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3) {
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
                "hasNextPage": true,
            },
            "edges": [
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
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
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &None,
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, order: DESC) {
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
                "hasNextPage": true,
            },
            "edges": [
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
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
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &None,
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, after: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
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
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
                {
                    "node": {
                        "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
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
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &None,
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
                Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, order: DESC, after: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
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
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
                    },
                },
                {
                    "node": {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
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
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(), MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")))),
                &OrderDirection::Ascending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
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
                    id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                },
            ])
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allMedia(first: 3, before: "MjAyMi0wNi0wMVQxMjozNDo1OC4wMDAwMDAAOTk5OTk5OTktOTk5OS05OTk5LTk5OTktOTk5OTk5OTk5OTk5") {
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
                        "createdAt": "2022-06-01T12:34:56",
                        "updatedAt": "2022-06-01T00:05:00",
                    },
                },
                {
                    "node": {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "createdAt": "2022-06-01T12:34:57",
                        "updatedAt": "2022-06-01T00:05:01",
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
        .withf(|tag_depth, replicas, sources, since, until, order, limit| {
            (tag_depth, replicas, sources, since, until, order, limit) == (
                &None,
                &false,
                &false,
                &None,
                &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(), MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")))),
                &OrderDirection::Descending,
                &4,
            )
        })
        .returning(|_, _, _, _, _, _, _| {
            Ok(vec![
                Medium {
                    id: MediumId::from(uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 59)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 3)).unwrap(),
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
            allMedia(first: 3, order: DESC, before: "MjAyMi0wNi0wMVQxMjozNDo1Ny4wMDAwMDAAODg4ODg4ODgtODg4OC04ODg4LTg4ODgtODg4ODg4ODg4ODg4") {
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
                        "createdAt": "2022-06-01T12:34:59",
                        "updatedAt": "2022-06-01T00:05:03",
                    },
                },
                {
                    "node": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "createdAt": "2022-06-01T12:34:58",
                        "updatedAt": "2022-06-01T00:05:02",
                    },
                },
            ],
        },
    }));
}
