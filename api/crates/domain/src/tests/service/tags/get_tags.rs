use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::tags::{Tag, TagDepth, TagId},
    error::{Error, ErrorKind},
    repository::{Direction, Order},
    service::tags::{TagsService, TagsServiceInterface},
};

use super::mocks::domain::repository::{tag_types::MockTagTypesRepository, tags::MockTagsRepository};

#[tokio::test]
async fn succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_all()
        .times(1)
        .withf(|depth, root, after, before, order, limit| {
            (depth, root, after, before, order, limit) == (
                &TagDepth::new(0, 1),
                &false,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Box::pin(ok(vec![
                Tag {
                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: vec![
                        Tag {
                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            name: "赤座あかり".to_string(),
                            kana: "あかざあかり".to_string(),
                            aliases: Default::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            name: "歳納京子".to_string(),
                            kana: "としのうきょうこ".to_string(),
                            aliases: Default::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    name: "赤座あかり".to_string(),
                    kana: "あかざあかり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                },
            ]))
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags(TagDepth::new(0, 1), false, None, Order::Ascending, Direction::Forward, 10).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: Default::default(),
            parent: None,
            children: vec![
                Tag {
                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    name: "赤座あかり".to_string(),
                    kana: "あかざあかり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 1, 0).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
            name: "歳納京子".to_string(),
            kana: "としのうきょうこ".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 2, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 3, 0).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_all()
        .times(1)
        .withf(|depth, root, after, before, order, limit| {
            (depth, root, after, before, order, limit) == (
                &TagDepth::new(0, 1),
                &false,
                &None,
                &Order::Ascending,
                &Direction::Forward,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _| Box::pin(err(Error::other("error communicating with database"))));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags(TagDepth::new(0, 1), false, None, Order::Ascending, Direction::Forward, 10).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
