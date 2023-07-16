use std::collections::BTreeSet;

use anyhow::anyhow;
use chrono::NaiveDate;
use domain::{
    entity::{
        tag_types::{TagType, TagTypeId},
        tags::{AliasSet, Tag, TagDepth, TagId},
    },
    repository::{
        tag_types::MockTagTypesRepository,
        tags::MockTagsRepository,
        DeleteResult, OrderDirection,
    },
    service::tags::{TagsService, TagsServiceInterface},
};
use pretty_assertions::assert_eq;
use uuid::uuid;

#[tokio::test]
async fn create_tag_succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_create()
        .times(1)
        .withf(|name, kana, aliases, parent_id, depth| {
            (name, kana, aliases, parent_id, depth) == (
                "赤座あかり",
                "あかざあかり",
                &["アッカリーン".to_string()],
                &Some(TagId::from(uuid!("22222222-2222-2222-2222-222222222222"))),
                &TagDepth::new(1, 1),
            )
        })
        .returning(|_, _, _, _, _| {
            Ok(Tag {
                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
            })
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.create_tag(
        "赤座あかり",
        "あかざあかり",
        &["アッカリーン".to_string()],
        Some(TagId::from(uuid!("22222222-2222-2222-2222-222222222222"))),
        TagDepth::new(1, 1),
    ).await.unwrap();

    assert_eq!(actual, Tag {
        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        name: "赤座あかり".to_string(),
        kana: "あかざあかり".to_string(),
        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
        parent: Some(Box::new(Tag {
            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        })),
        children: Vec::new(),
        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
    });
}

#[tokio::test]
async fn create_tag_fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_create()
        .times(1)
        .withf(|name, kana, aliases, parent_id, depth| {
            (name, kana, aliases, parent_id, depth) == (
                "赤座あかり",
                "あかざあかり",
                &["アッカリーン".to_string()],
                &Some(TagId::from(uuid!("22222222-2222-2222-2222-222222222222"))),
                &TagDepth::new(1, 1),
            )
        })
        .returning(|_, _, _, _, _| Err(anyhow!("error creating a tag")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.create_tag(
        "赤座あかり",
        "あかざあかり",
        &["アッカリーン".to_string()],
        Some(TagId::from(uuid!("22222222-2222-2222-2222-222222222222"))),
        TagDepth::new(1, 1),
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn create_tag_type_succeeds() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_create()
        .times(1)
        .withf(|slug, name| (slug, name) == ("character", "キャラクター"))
        .returning(|_, _| {
            Ok(TagType {
                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                slug: "character".to_string(),
                name: "キャラクター".to_string(),
            })
        });

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.create_tag_type("character", "キャラクター").await.unwrap();

    assert_eq!(actual, TagType {
        id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
        slug: "character".to_string(),
        name: "キャラクター".to_string(),
    })
}

#[tokio::test]
async fn create_tag_type_fails() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_create()
        .times(1)
        .withf(|slug, name| (slug, name) == ("character", "キャラクター"))
        .returning(|_, _| Err(anyhow!("error creating a tag type")));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.create_tag_type("character", "キャラクター").await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_tags_succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_all()
        .times(1)
        .withf(|depth, root, after, before, order, limit| {
            (depth, root, after, before, order, limit) == (
                &TagDepth::new(0, 1),
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _| {
            Ok(vec![
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
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            name: "歳納京子".to_string(),
                            kana: "としのうきょうこ".to_string(),
                            aliases: Default::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    name: "赤座あかり".to_string(),
                    kana: "あかざあかり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                },
            ])
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags(TagDepth::new(0, 1), false, None, None, OrderDirection::Ascending, 10).await.unwrap();

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
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
            name: "歳納京子".to_string(),
            kana: "としのうきょうこ".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn get_tags_fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_all()
        .times(1)
        .withf(|depth, root, after, before, order, limit| {
            (depth, root, after, before, order, limit) == (
                &TagDepth::new(0, 1),
                &false,
                &None,
                &None,
                &OrderDirection::Ascending,
                &10,
            )
        })
        .returning(|_, _, _, _, _, _| Err(anyhow!("error fetching tags")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags(TagDepth::new(0, 1), false, None, None, OrderDirection::Ascending, 10).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_tags_by_ids_succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, depth| {
            (ids, depth) == (
                &[
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                ],
                &TagDepth::new(0, 1),
            )
        })
        .returning(|_, _| {
            Ok(vec![
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
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            name: "歳納京子".to_string(),
                            kana: "としのうきょうこ".to_string(),
                            aliases: Default::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                },
            ])
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags_by_ids(
        [
            TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
        ],
        TagDepth::new(0, 1),
    ).await.unwrap();

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
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
            name: "歳納京子".to_string(),
            kana: "としのうきょうこ".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn get_tags_by_ids_fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids, depth| {
            (ids, depth) == (
                &[
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                ],
                &TagDepth::new(0, 1),
            )
        })
        .returning(|_, _| Err(anyhow!("error fetching the tags")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags_by_ids(
        [
            TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
        ],
        TagDepth::new(0, 1),
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_tags_by_name_or_alias_like_succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_by_name_or_alias_like()
        .times(1)
        .withf(|name_or_alias_like, depth| (name_or_alias_like, depth) == ("り", &TagDepth::new(0, 1)))
        .returning(|_, _| {
            Ok(vec![
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
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            name: "歳納京子".to_string(),
                            kana: "としのうきょうこ".to_string(),
                            aliases: Default::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                        },
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    name: "赤座あかり".to_string(),
                    kana: "あかざあかり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
            ])
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags_by_name_or_alias_like("り", TagDepth::new(0, 1)).await.unwrap();

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
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        },
    ]);
}

#[tokio::test]
async fn get_tags_by_name_or_alias_like_fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_fetch_by_name_or_alias_like()
        .times(1)
        .withf(|name_or_alias_like, depth| (name_or_alias_like, depth) == ("り", &TagDepth::new(0, 1)))
        .returning(|_, _| Err(anyhow!("error fetching the tags")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tags_by_name_or_alias_like("り", TagDepth::new(0, 1)).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn get_tag_types_succeeds() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| {
            Ok(vec![
                TagType {
                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                },
                TagType {
                    id: TagTypeId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    slug: "illustrator".to_string(),
                    name: "イラストレーター".to_string(),
                },
            ])
        });

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tag_types().await.unwrap();

    assert_eq!(actual, vec![
        TagType {
            id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            slug: "character".to_string(),
            name: "キャラクター".to_string(),
        },
        TagType {
            id: TagTypeId::from(uuid!("55555555-5555-5555-5555-555555555555")),
            slug: "illustrator".to_string(),
            name: "イラストレーター".to_string(),
        },
    ]);
}

#[tokio::test]
async fn get_tag_types_fails() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| Err(anyhow!("error fetching the tag types")));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.get_tag_types().await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn update_tag_by_id_succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, name, kana, add_aliases: &Vec<_>, remove_aliases: &Vec<_>, depth| {
            (id, name, kana, add_aliases, remove_aliases, depth) == (
                &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &Some("赤座あかり".to_string()),
                &Some("あかざあかり".to_string()),
                &vec!["アッカリーン".to_string()],
                &vec![],
                &TagDepth::new(0, 1),
            )
        })
        .returning(|_, _, _, _, _, _| {
            Ok(Tag {
                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
            })
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.update_tag_by_id(
        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        Some("赤座あかり".to_string()),
        Some("あかざあかり".to_string()),
        vec!["アッカリーン".to_string()],
        vec![],
        TagDepth::new(0, 1),
    ).await.unwrap();

    assert_eq!(actual, Tag {
        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        name: "赤座あかり".to_string(),
        kana: "あかざあかり".to_string(),
        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
        parent: Some(Box::new(Tag {
            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        })),
        children: Vec::new(),
        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
    });
}

#[tokio::test]
async fn update_tag_by_id_fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, name, kana, add_aliases: &Vec<_>, remove_aliases: &Vec<_>, depth| {
            (id, name, kana, add_aliases, remove_aliases, depth) == (
                &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &Some("赤座あかり".to_string()),
                &Some("あかざあかり".to_string()),
                &vec!["アッカリーン".to_string()],
                &vec![],
                &TagDepth::new(0, 1),
            )
        })
        .returning(|_, _, _, _, _, _| Err(anyhow!("error updating the tag")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.update_tag_by_id(
        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        Some("赤座あかり".to_string()),
        Some("あかざあかり".to_string()),
        vec!["アッカリーン".to_string()],
        vec![],
        TagDepth::new(0, 1),
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn update_tag_type_by_id_succeeds() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, slug, name| {
            (id, slug, name) == (
                &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                &Some("characters"),
                &None,
            )
        })
        .returning(|_, _, _| {
            Ok(TagType {
                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                slug: "characters".to_string(),
                name: "キャラクター".to_string(),
            })
        });

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.update_tag_type_by_id(
        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
        Some("characters"),
        None,
    ).await.unwrap();

    assert_eq!(actual, TagType {
        id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
        slug: "characters".to_string(),
        name: "キャラクター".to_string(),
    });
}

#[tokio::test]
async fn update_tag_type_by_id_fails() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, slug, name| {
            (id, slug, name) == (
                &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                &Some("characters"),
                &None,
            )
        })
        .returning(|_, _, _| Err(anyhow!("error updating the tag type")));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.update_tag_type_by_id(
        TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
        Some("characters"),
        None,
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn attach_tag_by_id_succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_attach_by_id()
        .times(1)
        .withf(|id, parent_id, depth| {
            (id, parent_id, depth) == (
                &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                &TagDepth::new(1, 1),
            )
        })
        .returning(|_, _, _| {
            Ok(Tag {
                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: Default::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
            })
        });

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.attach_tag_by_id(
        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        TagDepth::new(1, 1),
    ).await.unwrap();

    assert_eq!(actual, Tag {
        id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        name: "赤座あかり".to_string(),
        kana: "あかざあかり".to_string(),
        aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
        parent: Some(Box::new(Tag {
            id: TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: Default::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        })),
        children: Vec::new(),
        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
    });
}

#[tokio::test]
async fn attach_tag_by_id_fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_attach_by_id()
        .times(1)
        .withf(|id, parent_id, depth| {
            (id, parent_id, depth) == (
                &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                &TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                &TagDepth::new(1, 1),
            )
        })
        .returning(|_, _, _| Err(anyhow!("error attaching the tag")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.attach_tag_by_id(
        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
        TagDepth::new(1, 1),
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn detach_tag_by_id_succeeds() {
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
            Ok(Tag {
                id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["アッカリーン".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
            })
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
        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
    });
}

#[tokio::test]
async fn detach_tag_by_id_fails() {
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
        .returning(|_, _| Err(anyhow!("error detaching the tag")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.detach_tag_by_id(
        TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        TagDepth::new(1, 1),
    ).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn delete_tag_by_id_succeeds() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id, recursive| id == &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")) && *recursive)
        .returning(|_, _| Ok(DeleteResult::Deleted(1)));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_by_id(TagId::from(uuid!("33333333-3333-3333-3333-333333333333")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_tag_by_id_fails() {
    let mut mock_tags_repository = MockTagsRepository::new();
    mock_tags_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id, recursive| id == &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")) && *recursive)
        .returning(|_, _| Err(anyhow!("error deleting the tag")));

    let mock_tag_types_repository = MockTagTypesRepository::new();

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_by_id(TagId::from(uuid!("33333333-3333-3333-3333-333333333333")), true).await;

    assert!(actual.is_err());
}

#[tokio::test]
async fn delete_tag_type_by_id_succeeds() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")))
        .returning(|_| Ok(DeleteResult::Deleted(1)));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_type_by_id(TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444"))).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_tag_type_by_id_fails() {
    let mock_tags_repository = MockTagsRepository::new();
    let mut mock_tag_types_repository = MockTagTypesRepository::new();
    mock_tag_types_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")))
        .returning(|_| Err(anyhow!("error deleting the tag type")));

    let service = TagsService::new(mock_tags_repository, mock_tag_types_repository);
    let actual = service.delete_tag_type_by_id(TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444"))).await;

    assert!(actual.is_err());
}
