use anyhow::anyhow;
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::tags::TagId,
    error::{Error, ErrorKind},
    repository::DeleteResult,
    service::tags::{TagsService, TagsServiceInterface},
};

use super::mocks::domain::repository::{tag_types::MockTagTypesRepository, tags::MockTagsRepository};

#[tokio::test]
async fn succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id, recursive| id == &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")) && *recursive)
        .returning(|_, _| Box::pin(ok(DeleteResult::Deleted(1))));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_by_id(TagId::from(uuid!("33333333-3333-3333-3333-333333333333")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id, recursive| id == &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")) && *recursive)
        .returning(|_, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_by_id(TagId::from(uuid!("33333333-3333-3333-3333-333333333333")), true).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
