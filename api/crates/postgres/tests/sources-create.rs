use domain::{
    entity::external_services::{ExternalService, ExternalServiceId, ExternalMetadata},
    repository::sources::SourcesRepository,
};
use postgres::sources::PostgresSourcesRepository;
use pretty_assertions::assert_eq;
use serde_json::json;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
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
            name: "pixiv".to_string(),
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Pixiv { id: 123456789 });

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
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
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
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
            name: "Custom".to_string(),
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Custom(r#"{"id":123456789}"#.to_string()));

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
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
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
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
            name: "Custom".to_string(),
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Custom(r#""123456789abcdefg""#.to_string()));

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14"));
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata"),
        json!("123456789abcdefg"),
    );
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
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
            name: "Custom".to_string(),
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Custom("123456789".to_string()));

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14"));
    assert_eq!(
        actual.get::<serde_json::Value, &str>("external_metadata"),
        json!(123456789),
    );
}
