use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
        sources::SourceId,
    },
    error::ErrorKind,
    repository::sources::SourcesRepository,
};
use postgres::sources::PostgresSourcesRepository;
use pretty_assertions::{assert_eq, assert_matches};
use serde_json::json;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_external_metadata_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")),
        None,
        Some(ExternalMetadata::Pixiv { id: 123456789 }),
    ).await.unwrap();

    assert_eq!(actual.id, SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")));
    assert_eq!(
        actual.external_service,
        ExternalService {
            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            slug: "pixiv".to_string(),
            kind: ExternalServiceKind::Pixiv,
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Pixiv { id: 123456789 });
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 14).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata", "external_metadata_extra" FROM "sources" WHERE "id" = $1"#)
        .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"));
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata"),
        json!({
            "type": "pixiv",
            "id": 123456789,
        }),
    );
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata_extra"),
        json!({
            "type": "pixiv",
        }),
    );
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_external_service_and_external_metadata_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")),
        Some(ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7"))),
        Some(ExternalMetadata::Skeb { id: 7777, creator_id: "creator_03".to_string() }),
    ).await.unwrap();

    assert_eq!(actual.id, SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")));
    assert_eq!(
        actual.external_service,
        ExternalService {
            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
            slug: "skeb".to_string(),
            kind: ExternalServiceKind::Skeb,
            name: "Skeb".to_string(),
            base_url: Some("https://skeb.jp".to_string()),
            url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
        },
    );
    assert_eq!(
        actual.external_metadata,
        ExternalMetadata::Skeb {
            id: 7777,
            creator_id: "creator_03".to_string(),
        },
    );
    assert_eq!(actual.created_at, Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap());
    assert_ne!(actual.updated_at, Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 14).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata", "external_metadata_extra" FROM "sources" WHERE "id" = $1"#)
        .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7"));
    assert_eq!(actual.get::<serde_json::Value, &str>("external_metadata"), json!({
        "type": "skeb",
        "id": 7777,
        "creatorId": "creator_03",
    }));
    assert_eq!(actual.get::<serde_json::Value, &str>("external_metadata_extra"), json!({
        "type": "skeb",
    }));
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        None,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::SourceNotFound { id } if id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")));
}
