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
        .expect_update_by_id()
        .times(1)
        .withf(|id, slug, name, kana| {
            (id, slug, name, kana) == (
                &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                &Some("characters"),
                &None,
                &None,
            )
        })
        .returning(|_, _, _, _| {
            Box::pin(ok(TagType {
                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                slug: "characters".to_string(),
                name: "キャラクター".to_string(),
                kana: "キャラクター".to_string(),
            }))
        });

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.update_tag_type_by_id(
        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
        Some("characters"),
        None,
        None,
    ).await.unwrap();

    assert_eq!(actual, TagType {
        id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
        slug: "characters".to_string(),
        name: "キャラクター".to_string(),
        kana: "キャラクター".to_string(),
    });
}

#[tokio::test]
async fn fails() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, slug, name, kana| {
            (id, slug, name, kana) == (
                &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                &Some("characters"),
                &None,
                &None,
            )
        })
        .returning(|_, _, _, _| Box::pin(err(Error::other("error communicating with database"))));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.update_tag_type_by_id(
        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
        Some("characters"),
        None,
        None,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
