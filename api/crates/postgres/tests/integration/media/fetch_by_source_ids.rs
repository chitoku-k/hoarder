use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::{Medium, MediumId},
        sources::SourceId,
    },
    repository::{media::MediaRepository, Direction, Order},
};
use ordermap::OrderMap;
use postgres::media::PostgresMediaRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ].into_iter(),
        None,
        false,
        false,
        None,
        Order::Ascending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
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

#[test_context(DatabaseContext)]
#[tokio::test]
async fn desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ].into_iter(),
        None,
        false,
        false,
        None,
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn since_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ].into_iter(),
        None,
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Ascending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn since_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ].into_iter(),
        None,
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(), MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")))),
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn until_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ].into_iter(),
        None,
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        Order::Ascending,
        Direction::Backward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("ccc5717b-cf11-403d-b466-f37cf1c2e6f6")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
        Medium {
            id: MediumId::from(uuid!("2872ed9d-4db9-4b25-b86f-791ad009cc0a")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
async fn until_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresMediaRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_source_ids(
        [
            SourceId::from(uuid!("8939ee67-5fb8-4204-a496-bb570a952f8b")),
            SourceId::from(uuid!("435d422e-acd0-4b22-b46c-180894a91049")),
            SourceId::from(uuid!("76a94241-1736-4823-bb59-bef097c687e1")),
        ].into_iter(),
        None,
        false,
        false,
        Some((Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(), MediumId::from(uuid!("6356503d-6ab6-4e39-bb86-3311219c7fd1")))),
        Order::Descending,
        Direction::Backward,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Medium {
            id: MediumId::from(uuid!("02c4e79d-2d61-4277-9760-5596adf488ce")),
            sources: Vec::new(),
            tags: OrderMap::new(),
            replicas: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
    ]);
}
