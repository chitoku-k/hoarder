use std::collections::BTreeSet;

use domain::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    repository::{tags::TagsRepository, OrderDirection},
};
use chrono::NaiveDate;
use postgres::tags::PostgresTagsRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn root_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        None,
        OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
            name: "アークナイツ".to_string(),
            kana: "アークナイツ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
            name: "東方Project".to_string(),
            kana: "とうほうProject".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn root_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        None,
        OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
            name: "ブルーアーカイブ".to_string(),
            kana: "ブルーアーカイブ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
            name: "東方Project".to_string(),
            kana: "とうほうProject".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn root_after_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
            name: "ブルーアーカイブ".to_string(),
            kana: "ブルーアーカイブ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn root_after_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
            name: "ブルーアーカイブ".to_string(),
            kana: "ブルーアーカイブ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn root_before_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
            name: "アークナイツ".to_string(),
            kana: "アークナイツ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn root_before_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
            name: "アークナイツ".to_string(),
            kana: "アークナイツ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn no_root_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        None,
        OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
            name: "アークナイツ".to_string(),
            kana: "アークナイツ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn no_root_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        None,
        OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
            name: "鈴仙・優曇華院・イナバ".to_string(),
            kana: "れいせん・うどんげいん・いなば".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
            name: "吉川ちなつ".to_string(),
            kana: "よしかわちなつ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn no_root_after_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
            name: "歳納京子".to_string(),
            kana: "としのうきょうこ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            name: "七森中☆ごらく部".to_string(),
            kana: "ななもりちゅうごらくぶ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
            name: "博麗霊夢".to_string(),
            kana: "はくれいれいむ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn no_root_after_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
            name: "鈴仙・優曇華院・イナバ".to_string(),
            kana: "れいせん・うどんげいん・いなば".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
            name: "吉川ちなつ".to_string(),
            kana: "よしかわちなつ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn no_root_before_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
            name: "アークナイツ".to_string(),
            kana: "アークナイツ".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn no_root_before_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
            name: "古明地こいし".to_string(),
            kana: "こめいじこいし".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}
