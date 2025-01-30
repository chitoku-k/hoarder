use domain::{
    entity::tag_types::{TagType, TagTypeId},
    repository::tag_types::TagTypesRepository,
};
use postgres::tag_types::PostgresTagTypesRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

use super::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
async fn succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagTypesRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids([
        TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
        TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
    ].into_iter()).await.unwrap();

    assert_eq!(actual, vec![
        TagType {
            id: TagTypeId::from(uuid!("67738231-9b3a-4f45-94dc-1ba302e50e38")),
            slug: "character".to_string(),
            name: "キャラクター".to_string(),
            kana: "キャラクター".to_string(),
        },
        TagType {
            id: TagTypeId::from(uuid!("1e5021f0-d8ef-4859-815a-747bf3175724")),
            slug: "work".to_string(),
            name: "作品".to_string(),
            kana: "さくひん".to_string(),
        },
    ]);
}
