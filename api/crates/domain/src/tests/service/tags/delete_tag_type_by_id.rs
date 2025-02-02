use anyhow::anyhow;
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::tag_types::TagTypeId,
    error::{Error, ErrorKind},
    repository::DeleteResult,
    service::tags::{TagsService, TagsServiceInterface},
};

use super::mocks::domain::repository::{tag_types::MockTagTypesRepository, tags::MockTagsRepository};

#[tokio::test]
async fn succeeds() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_type_by_id(TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn fails() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_type_by_id(TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444"))).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
