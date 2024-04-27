use domain::{
    entity::external_services::{ExternalService, ExternalServiceId},
    repository::external_services::ExternalServicesRepository,
};
use postgres::external_services::PostgresExternalServicesRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all().await.unwrap();

    assert_eq!(actual, vec![
        ExternalService {
            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            slug: "pixiv".to_string(),
            kind: "pixiv".to_string(),
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("2018afa2-aed9-46de-af9e-02e5fab64ed7")),
            slug: "skeb".to_string(),
            kind: "skeb".to_string(),
            name: "Skeb".to_string(),
            base_url: Some("https://skeb.jp".to_string()),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab")),
            slug: "twitter".to_string(),
            kind: "twitter".to_string(),
            name: "Twitter".to_string(),
            base_url: Some("https://twitter.com".to_string()),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("6c07eb4d-93a1-4efd-afce-e13f8f2c0e14")),
            slug: "whatever".to_string(),
            kind: "custom".to_string(),
            name: "Custom".to_string(),
            base_url: None,
        },
    ]);
}
