use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
        sources::{Source, SourceId},
    },
    repository::sources::SourcesRepository,
};
use postgres::sources::PostgresSourcesRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_external_metadata(
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        ExternalMetadata::Pixiv { id: 8888888 },
    ).await.unwrap();

    assert_eq!(actual, Some(Source {
        id: SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            slug: "pixiv".to_string(),
            kind: ExternalServiceKind::Pixiv,
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
        },
        external_metadata: ExternalMetadata::Pixiv { id: 8888888 },
        created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 14).unwrap(),
    }));
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds_with_extra(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_external_metadata(
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        ExternalMetadata::X{ id: 222222222222, creator_id: Some("creator_01".to_string()) },
    ).await.unwrap();

    assert_eq!(actual, Some(Source {
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
    }));
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn not_found(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_external_metadata(
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        ExternalMetadata::Pixiv { id: 10000000 },
    ).await.unwrap();

    assert!(actual.is_none());
}
