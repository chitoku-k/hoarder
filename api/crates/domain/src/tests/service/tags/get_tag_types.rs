use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::tag_types::{TagType, TagTypeId},
    error::{Error, ErrorKind},
    service::tags::{TagsService, TagsServiceInterface},
};

use super::mocks::domain::repository::{tag_types::MockTagTypesRepository, tags::MockTagsRepository};

#[tokio::test]
async fn succeeds() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| {
            Box::pin(ok(vec![
                TagType {
                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                    kana: "キャラクター".to_string(),
                },
                TagType {
                    id: TagTypeId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    slug: "illustrator".to_string(),
                    name: "イラストレーター".to_string(),
                    kana: "イラストレーター".to_string(),
                },
            ]))
        });

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tag_types().await.unwrap();

    assert_eq!(actual, vec![
        TagType {
            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            slug: "character".to_string(),
            name: "キャラクター".to_string(),
            kana: "キャラクター".to_string(),
        },
        TagType {
            id: TagTypeId::from(uuid!("55555555-5555-5555-5555-555555555555")),
            slug: "illustrator".to_string(),
            name: "イラストレーター".to_string(),
            kana: "イラストレーター".to_string(),
        },
    ]);
}

#[tokio::test]
async fn fails() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| Box::pin(err(Error::other("error communicating with database"))));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tag_types().await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
