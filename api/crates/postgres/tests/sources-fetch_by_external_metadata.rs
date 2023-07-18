use chrono::NaiveDate;
use domain::{
    entity::{
        external_services::{ExternalService, ExternalServiceId, ExternalMetadata},
        sources::{Source, SourceId},
    },
    repository::sources::SourcesRepository,
};
use postgres::sources::PostgresSourcesRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresSourcesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_external_metadata(
        ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
        ExternalMetadata::Pixiv { id: 8888888 },
    ).await.unwrap();

    assert_eq!(actual, Source {
        id: SourceId::from(uuid!("94055dd8-7a22-4137-b8eb-3a374df5e5d1")),
        external_service: ExternalService {
            id: ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3")),
            slug: "pixiv".to_string(),
            name: "pixiv".to_string(),
        },
        external_metadata: ExternalMetadata::Pixiv { id: 8888888 },
        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 14)).unwrap(),
    });
}
