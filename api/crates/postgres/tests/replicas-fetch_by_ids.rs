use chrono::NaiveDate;
use domain::{
    entity::replicas::{Replica, ReplicaId},
    repository::replicas::ReplicasRepository,
};
use postgres::replicas::PostgresReplicasRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresReplicasRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids([
        ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
        ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
        ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
    ]).await.unwrap();

    assert_eq!(actual, vec![
        Replica {
            id: ReplicaId::from(uuid!("1706c7bb-4152-44b2-9bbb-1179d09a19be")),
            display_order: Some(1),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/1706c7bb-4152-44b2-9bbb-1179d09a19be.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("6fae1497-e987-492e-987a-f9870b7d3c5b")),
            display_order: Some(2),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/6fae1497-e987-492e-987a-f9870b7d3c5b.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
        Replica {
            id: ReplicaId::from(uuid!("12ca56e2-6e77-43b9-9da9-9d968c80a1a5")),
            display_order: Some(3),
            has_thumbnail: false,
            original_url: "file:///var/lib/hoarder/12ca56e2-6e77-43b9-9da9-9d968c80a1a5.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}
