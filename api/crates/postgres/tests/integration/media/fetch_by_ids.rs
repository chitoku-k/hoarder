use chrono::{TimeZone, Utc};
use domain::{
    entity::media::{Medium, MediumId},
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
        false,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
    ]);
}
