use chrono::NaiveDate;
use domain::{
    entity::{
        external_services::{ExternalService, ExternalServiceId, ExternalMetadata},
        sources::SourceId,
    },
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
            name: "pixiv".to_string(),
        },
    );
    assert_eq!(actual.external_metadata, ExternalMetadata::Pixiv { id: 123456789 });
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 14)).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
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
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
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
            name: "Skeb".to_string(),
        },
    );
    assert_eq!(
        actual.external_metadata,
        ExternalMetadata::Skeb {
            id: 7777,
            creator_id: "creator_03".to_string(),
        },
    );
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 14)).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "external_service_id", "external_metadata" FROM "sources" WHERE "id" = $1"#)
        .bind(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<Uuid, &str>("external_service_id"), uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7"));
    assert_eq!(actual.get::<serde_json::Value, &str>("external_metadata"), json!({
        "type": "skeb",
        "id": 7777,
        "creator_id": "creator_03",
    }));
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fails(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        None,
    ).await;

    assert!(actual.is_err());
}
