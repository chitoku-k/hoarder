use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
        media::{Medium, MediumId},
        sources::{Source, SourceId},
    },
    repository::media::MediaRepository,
};
use ordermap::OrderMap;
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids(
        [
            MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
        ].into_iter(),
        None,
        false,
        true,
    ).await.unwrap();

    assert_eq!(actual, vec![
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
