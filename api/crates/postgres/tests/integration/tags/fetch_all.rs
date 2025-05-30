use domain::{
    entity::tags::{TagDepth, TagId},
    repository::{tags::TagsRepository, Direction, Order},
};
use postgres::tags::PostgresTagsRepository;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn with_out_of_bounds_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        Some(("".to_string(), TagId::from(uuid!("00000000-0000-0000-0000-000000000000")))),
        Order::Descending,
        Direction::Forward,
        3,
    ).await.unwrap();

    assert!(actual.is_empty());
}
