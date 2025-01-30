use domain::{
    entity::external_services::ExternalServiceId,
    repository::{external_services::ExternalServicesRepository, DeleteResult},
};
use postgres::external_services::PostgresExternalServicesRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresExternalServicesRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "external_services" WHERE "id" = $1"#)
        .bind(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(ExternalServiceId::from(uuid!("4e0c68c7-e5ec-4d60-b9eb-733f47290cd3"))).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
