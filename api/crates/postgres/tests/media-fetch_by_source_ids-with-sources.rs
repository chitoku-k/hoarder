use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::{Medium, MediumId},
        sources::{Source, SourceId},
    },
    repository::{media::MediaRepository, Direction, Order},
};
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        None,
        Order::Ascending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 12).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        None,
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 12).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Ascending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 4444444 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 12).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("5c872f82-2ad0-47c4-8c6f-64efc9443128")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        Order::Ascending,
        Direction::Backward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3e1150b0-144a-4fcf-a202-b93a5f3274db")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 2222222 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 1111111 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        Order::Descending,
        Direction::Backward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
    ]);
}
