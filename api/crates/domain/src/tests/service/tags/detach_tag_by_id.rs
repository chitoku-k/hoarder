use std::collections::BTreeSet;

use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    error::{Error, ErrorKind},
    service::tags::{TagsService, TagsServiceInterface},
};

use super::mocks::domain::repository::{tag_types::MockTagTypesRepository, tags::MockTagsRepository};

#[tokio::test]
async fn succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_detach_by_id()
        .times(1)
        .withf(|id, depth| {
            (id, depth) == (
                &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &TagDepth::new(1, 1),
            )
        })
        .returning(|_, _| {
            Box::pin(ok(Tag {
                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
            }))
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.detach_tag_by_id(
        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        TagDepth::new(1, 1),
    ).await.unwrap();

    assert_eq!(actual, Tag {
        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        name: "赤座あかり".to_string(),
        kana: "あかざあかり".to_string(),
        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
        parent: None,
        children: Vec::new(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
    });
}

#[tokio::test]
async fn fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_detach_by_id()
        .times(1)
        .withf(|id, depth| {
            (id, depth) == (
                &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &TagDepth::new(1, 1),
            )
        })
        .returning(|_, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.detach_tag_by_id(
        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        TagDepth::new(1, 1),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
