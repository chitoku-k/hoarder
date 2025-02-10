use domain::{
    entity::external_services::{ExternalMetadata, ExternalService, ExternalServiceId, ExternalServiceKind},
    repository::sources::SourcesRepository,
};
use postgres::sources::PostgresSourcesRepository;
use pretty_assertions::assert_eq;
use serde_json::json;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds_with_default(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.create(
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        ExternalMetadata::Pixiv { id: 123456789 },
    ).await.unwrap();

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

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata", "external_metadata_extra" FROM "sources" WHERE "id" = $1"#)
        .bind(*actual.id)
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
async fn succeeds_with_extra(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.create(
        ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
        ExternalMetadata::X{ id: 123456789, creator_id: Some("creator_01".to_string()) },
    ).await.unwrap();

    assert_eq!(
        actual.external_service,
        ExternalService {
            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
            slug: "x".to_string(),
            kind: ExternalServiceKind::X,
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::X { id: 123456789, creator_id: Some("creator_01".to_string()) });

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata", "external_metadata_extra" FROM "sources" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab"));
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata"),
        json!({
            "type": "x",
            "id": 123456789,
        }),
    );
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata_extra"),
        json!({
            "type": "x",
            "creatorId": "creator_01",
        }),
    );
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds_with_custom_object(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.create(
        ExternalServiceId::from(uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14")),
        ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string()),
    ).await.unwrap();

    assert_eq!(
        actual.external_service,
        ExternalService {
            id: ExternalServiceId::from(uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14")),
            slug: "whatever".to_string(),
            kind: ExternalServiceKind::Custom("custom".to_string()),
            name: "Custom".to_string(),
            base_url: None,
            url_pattern: None,
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string()));

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata", "external_metadata_extra" FROM "sources" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14"));
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata"),
        json!({
            "id": 123456789,
        }),
    );
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata_extra"),
        json!({}),
    );
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds_with_custom_string(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.create(
        ExternalServiceId::from(uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14")),
        ExternalMetadata::Custom(r#""123456789abcdefg""#.to_string()),
    ).await.unwrap();

    assert_eq!(
        actual.external_service,
        ExternalService {
            id: ExternalServiceId::from(uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14")),
            slug: "whatever".to_string(),
            kind: ExternalServiceKind::Custom("custom".to_string()),
            name: "Custom".to_string(),
            base_url: None,
            url_pattern: None,
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Custom(r#""123456789abcdefg""#.to_string()));

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata", "external_metadata_extra" FROM "sources" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14"));
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata"),
        json!("123456789abcdefg"),
    );
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata_extra"),
        json!({}),
    );
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds_with_custom_number(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.create(
        ExternalServiceId::from(uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14")),
        ExternalMetadata::Custom("123456789".to_string()),
    ).await.unwrap();

    assert_eq!(
        actual.external_service,
        ExternalService {
            id: ExternalServiceId::from(uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14")),
            slug: "whatever".to_string(),
            kind: ExternalServiceKind::Custom("custom".to_string()),
            name: "Custom".to_string(),
            base_url: None,
            url_pattern: None,
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Custom("123456789".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata", "external_metadata_extra" FROM "sources" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14"));
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata"),
        json!(123456789),
    );
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata_extra"),
        json!({}),
    );
}
