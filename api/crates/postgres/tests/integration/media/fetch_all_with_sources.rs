use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
        media::{Medium, MediumId},
        sources::{Source, SourceId},
    },
    repository::{media::MediaRepository, Direction, Order},
};
use ordermap::OrderMap;
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::Skeb,
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                        url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
async fn desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
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
            id: MediumId::from(uuid!("616aa868-dff8-4a59-b79c-58469114b380")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("a8c1a9d2-0d17-422b-9c02-632cb7712b5b")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("e607c6f5-af17-4f65-9868-b3e72f692f4d")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 5555555 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 13).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("2a82a031-e27a-443e-9f22-bb190f70633a")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        kind: ExternalServiceKind::Skeb,
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                        url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 4444, creator_id: "creator_01".to_string() },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("da2e3cc8-5b12-45fc-b720-815e74fb8fe6")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 6666666 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("725792bf-dbf0-4af1-b639-a147f0b327b2")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
                        slug: "skeb".to_string(),
                        kind: ExternalServiceKind::Skeb,
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                        url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Skeb { id: 2222, creator_id: "creator_02".to_string() },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
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
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 16).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 111111111111, creator_id: Some("creator_01".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 17).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            tags: OrderMap::new(),
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
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        None,
        false,
        true,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(), MediumId::from(uuid!("348ffaa9-624b-488f-9c63-d61f78db06a7")))),
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("fdd88e21-fbb0-49e0-854e-54f2cca208f1")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("ddfe5d3b-17ff-4f22-ae25-042f534b84a9")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 9999999 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
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
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: OrderMap::new(),
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::Skeb,
                        name: "Skeb".to_string(),
                        base_url: Some("https://skeb.jp".to_string()),
                        url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
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
async fn until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
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
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 222222222222, creator_id: None },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 14).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 15).unwrap(),
                },
            ],
            tags: OrderMap::new(),
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
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
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
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 333333333333, creator_id: Some("creator_03".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 16).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("43b77865-c05d-4733-b336-95b5522a8a46")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("3a8f9940-08bc-48bf-a6dd-e9ceaf685dfd")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
                        slug: "pixiv".to_string(),
                        kind: ExternalServiceKind::Pixiv,
                        name: "pixiv".to_string(),
                        base_url: Some("https://www.pixiv.net".to_string()),
                        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 7777777 },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 16).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("082bdad0-46a9-4637-af44-3c91a605a5f1")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
                        slug: "x".to_string(),
                        kind: ExternalServiceKind::X,
                        name: "X".to_string(),
                        base_url: Some("https://x.com".to_string()),
                        url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                    },
                    external_metadata: ExternalMetadata::X { id: 111111111111, creator_id: Some("creator_01".to_string()) },
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 15).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 17).unwrap(),
                },
            ],
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}
