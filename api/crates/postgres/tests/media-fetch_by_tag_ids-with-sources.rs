use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
        media::{Medium, MediumId},
        sources::{Source, SourceId},
        tag_types::TagTypeId,
        tags::TagId,
    },
    repository::{media::MediaRepository, Direction, Order},
};
use ordermap::OrderMap;
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
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
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
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        kind: "skeb".to_string(),
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
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
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
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
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 111111111111, creator_id: Some("creator_01".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 17).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 16).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        kind: "skeb".to_string(),
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(), MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")))),
        Order::Ascending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        kind: "skeb".to_string(),
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
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
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 111111111111, creator_id: Some("creator_01".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 17).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 16).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")))),
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
            tags: OrderMap::new(),
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
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
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
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("21cb17ac-6e4c-4da7-8b8a-bc17bc258196")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3d67f432-5ec0-44b6-a1a5-9034e5300351")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 3333333 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("4778ae3d-090e-4224-abc0-1a6c247801bd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        kind: "skeb".to_string(),
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 3333, creator_id: "creator_01".to_string() },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 11).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_tag_ids(
        [
            (
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            ),
        ],
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Descending,
        Direction::Backward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: "x".to_string(),
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 111111111111, creator_id: Some("creator_01".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 17).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 16).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}
