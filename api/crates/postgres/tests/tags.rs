use std::collections::BTreeSet;

use domain::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    repository::{self, tags::TagsRepository, DeleteResult},
};
use chrono::NaiveDate;
use futures::TryStreamExt;
use postgres::tags::PostgresTagsRepository;
use pretty_assertions::assert_eq;
use sqlx::Row;
use test_context::test_context;
use uuid::{uuid, Uuid};

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn create_with_parent_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.create(
        "七森中☆生徒会",
        "ななもりちゅうせいとかい",
        &["生徒会".to_string(), "七森中生徒会".to_string()],
        Some(TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))),
        TagDepth::new(2, 2),
    ).await.unwrap();

    let actual_id = actual.id;
    assert_eq!(actual.name, "七森中☆生徒会".to_string());
    assert_eq!(actual.kana, "ななもりちゅうせいとかい");
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["生徒会".to_string(), "七森中生徒会".to_string()])));
    assert_eq!(
        actual.parent,
        Some(Box::new(Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        })),
    );
    assert_eq!(actual.children, Vec::new());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "七森中☆生徒会");
    assert_eq!(actual.get::<&str, &str>("kana"), "ななもりちゅうせいとかい");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["生徒会".to_string(), "七森中生徒会".to_string()]);

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" = $1 ORDER BY "distance" DESC"#)
        .bind(*actual_id)
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 3);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[0].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[1].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), *actual_id);
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[2].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn create_without_parent_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.create(
        "七森中☆生徒会",
        "ななもりちゅうせいとかい",
        &["生徒会".to_string(), "七森中生徒会".to_string()],
        None,
        TagDepth::new(2, 2),
    ).await.unwrap();

    let actual_id = actual.id;
    assert_eq!(actual.name, "七森中☆生徒会".to_string());
    assert_eq!(actual.kana, "ななもりちゅうせいとかい");
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["生徒会".to_string(), "七森中生徒会".to_string()])));
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, Vec::new());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(*actual.id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "七森中☆生徒会");
    assert_eq!(actual.get::<&str, &str>("kana"), "ななもりちゅうせいとかい");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["生徒会".to_string(), "七森中生徒会".to_string()]);

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" = $1 ORDER BY "distance" DESC"#)
        .bind(*actual_id)
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 2);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[0].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), *actual_id);
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), *actual_id);
    assert_eq!(actual[1].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_ids_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_ids(
        [
            TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
            TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
        ],
        TagDepth::new(2, 2),
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
            name: "魔女っ娘ミラクるん".to_string(),
            kana: "まじょっこミラクるん".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: vec![
                Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: vec![
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
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_name_or_alias_like_with_name_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_name_or_alias_like("り", TagDepth::new(2, 2)).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
            name: "赤座あかり".to_string(),
            kana: "あかざあかり".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: vec![
                Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: vec![
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
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_name_or_alias_like_with_alias_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_name_or_alias_like("げ", TagDepth::new(2, 2)).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
            name: "鈴仙・優曇華院・イナバ".to_string(),
            kana: "れいせん・うどんげいん・いなば".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_by_name_or_alias_like_with_name_and_alias_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_name_or_alias_like("ん", TagDepth::new(2, 2)).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
            name: "鈴仙・優曇華院・イナバ".to_string(),
            kana: "れいせん・うどんげいん・いなば".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
            name: "魔女っ娘ミラクるん".to_string(),
            kana: "まじょっこミラクるん".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_depth_and_root_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        None,
        None,
        repository::OrderDirection::Ascending,
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
            children: vec![
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
                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                    name: "博麗霊夢".to_string(),
                    kana: "はくれいれいむ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    name: "フランドール・スカーレット".to_string(),
                    kana: "フランドール・スカーレット".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
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
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_depth_and_root_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        None,
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: vec![
                Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: vec![
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
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
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
            children: vec![
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
                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                    name: "博麗霊夢".to_string(),
                    kana: "はくれいれいむ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    name: "フランドール・スカーレット".to_string(),
                    kana: "フランドール・スカーレット".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                },
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
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_depth_and_root_after_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Ascending,
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
            children: vec![
                Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: vec![
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
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_depth_and_root_after_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: vec![
                Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: vec![
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
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
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
async fn fetch_all_with_depth_and_root_before_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Ascending,
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
async fn fetch_all_with_depth_and_root_before_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Descending,
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
async fn fetch_all_with_depth_and_no_root_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        false,
        None,
        None,
        repository::OrderDirection::Ascending,
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
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
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
async fn fetch_all_with_depth_and_no_root_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        false,
        None,
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
            name: "鈴仙・優曇華院・イナバ".to_string(),
            kana: "れいせん・うどんげいん・いなば".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
            name: "吉川ちなつ".to_string(),
            kana: "よしかわちなつ".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
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
            children: vec![
                Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: vec![
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
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_depth_and_no_root_after_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        false,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Ascending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
            name: "歳納京子".to_string(),
            kana: "としのうきょうこ".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            name: "七森中☆ごらく部".to_string(),
            kana: "ななもりちゅうごらくぶ".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            })),
            children: vec![
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
                    id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                    name: "船見結衣".to_string(),
                    kana: "ふなみゆい".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
            name: "博麗霊夢".to_string(),
            kana: "はくれいれいむ".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_depth_and_no_root_after_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        false,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
            name: "鈴仙・優曇華院・イナバ".to_string(),
            kana: "れいせん・うどんげいん・いなば".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
        Tag {
            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
            name: "吉川ちなつ".to_string(),
            kana: "よしかわちなつ".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
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
            children: vec![
                Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: vec![
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
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
                    ],
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                },
            ],
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_depth_and_no_root_before_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        false,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Ascending,
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
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
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
async fn fetch_all_with_depth_and_no_root_before_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        false,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
            name: "古明地こいし".to_string(),
            kana: "こめいじこいし".to_string(),
            aliases: AliasSet::default(),
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            })),
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
            parent: Some(Box::new(Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            })),
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_no_depth_and_root_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        None,
        repository::OrderDirection::Ascending,
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
async fn fetch_all_with_no_depth_and_root_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        None,
        repository::OrderDirection::Descending,
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
async fn fetch_all_with_no_depth_and_root_after_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Ascending,
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
async fn fetch_all_with_no_depth_and_root_after_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Descending,
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
async fn fetch_all_with_no_depth_and_root_before_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Ascending,
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
async fn fetch_all_with_no_depth_and_root_before_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        true,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Descending,
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
async fn fetch_all_with_no_depth_and_no_root_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        None,
        repository::OrderDirection::Ascending,
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
async fn fetch_all_with_no_depth_and_no_root_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        None,
        repository::OrderDirection::Descending,
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
async fn fetch_all_with_no_depth_and_no_root_after_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Ascending,
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
async fn fetch_all_with_no_depth_and_no_root_after_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        None,
        repository::OrderDirection::Descending,
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
async fn fetch_all_with_no_depth_and_no_root_before_asc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Ascending,
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
async fn fetch_all_with_no_depth_and_no_root_before_desc_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(0, 0),
        false,
        None,
        Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
        repository::OrderDirection::Descending,
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

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn fetch_all_with_out_of_bounds_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_all(
        TagDepth::new(2, 2),
        true,
        None,
        Some(("".to_string(), TagId::from(uuid!("00000000-0000-0000-0000-000000000000")))),
        repository::OrderDirection::Descending,
        3,
    ).await.unwrap();

    assert!(actual.is_empty());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_with_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        Some("ごらく部".to_string()),
        Some("ごらくぶ".to_string()),
        ["七森中☆ごらく部".to_string()],
        [],
        TagDepth::new(2, 2),
    ).await.unwrap();

    assert_eq!(actual.name, "ごらく部".to_string());
    assert_eq!(actual.kana, "ごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["七森中☆ごらく部".to_string()])));
    assert_eq!(actual.parent, Some(Box::new(Tag {
        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
        name: "ゆるゆり".to_string(),
        kana: "ゆるゆり".to_string(),
        aliases: AliasSet::default(),
        parent: None,
        children: Vec::new(),
        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
    })));
    assert_eq!(actual.children, vec![
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
            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
            name: "船見結衣".to_string(),
            kana: "ふなみゆい".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
    ]);
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "ごらく部");
    assert_eq!(actual.get::<&str, &str>("kana"), "ごらくぶ");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["七森中☆ごらく部".to_string()]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_without_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        Some("ごらく部".to_string()),
        Some("ごらくぶ".to_string()),
        ["七森中☆ごらく部".to_string()],
        [],
        TagDepth::new(0, 0),
    ).await.unwrap();

    assert_eq!(actual.name, "ごらく部".to_string());
    assert_eq!(actual.kana, "ごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["七森中☆ごらく部".to_string()])));
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
    assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

    let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(actual.get::<&str, &str>("name"), "ごらく部");
    assert_eq!(actual.get::<&str, &str>("kana"), "ごらくぶ");
    assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["七森中☆ごらく部".to_string()]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        None,
        [],
        [],
        TagDepth::new(0, 0),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn update_by_id_root_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.update_by_id(
        TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
        None,
        None,
        [],
        [],
        TagDepth::new(0, 0),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn attach_by_id_with_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(2, 2),
    ).await.unwrap();

    assert_eq!(actual.name, "七森中☆ごらく部".to_string());
    assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::default());
    assert_eq!(actual.parent, Some(Box::new(Tag {
        id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        name: "東方Project".to_string(),
        kana: "とうほうProject".to_string(),
        aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
        parent: None,
        children: Vec::new(),
        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
    })));
    assert_eq!(actual.children, vec![
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
            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
            name: "船見結衣".to_string(),
            kana: "ふなみゆい".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
    ]);
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
    assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6, $7) ORDER BY "descendant_id", "distance" DESC"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 23);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[0].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[1].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[4].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[8].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[14].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[15].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[16].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[16].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[16].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[17].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[18].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[18].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[18].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[19].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[19].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[19].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[20].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[21].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[21].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[21].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[22].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn attach_by_id_without_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(0, 0),
    ).await.unwrap();

    assert_eq!(actual.name, "七森中☆ごらく部".to_string());
    assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::default());
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
    assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6, $7) ORDER BY "descendant_id", "distance" DESC"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 23);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[0].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[1].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[3].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[4].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[8].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[9].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[14].get::<i32, &str>("distance"), 3);

    assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[15].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[16].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[16].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[16].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[17].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[17].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[18].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[18].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[18].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[19].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[19].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[19].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[20].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[20].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[21].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[21].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[21].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[22].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
    assert_eq!(actual[22].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn attach_by_id_root_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(0, 0),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn attach_by_id_non_existing_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.attach_by_id(
        TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
        TagDepth::new(0, 0),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn detach_by_id_with_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.detach_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        TagDepth::new(2, 2),
    ).await.unwrap();

    assert_eq!(actual.name, "七森中☆ごらく部".to_string());
    assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::default());
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, vec![
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
            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
            name: "船見結衣".to_string(),
            kana: "ふなみゆい".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
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
    ]);
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
    assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6) ORDER BY "descendant_id", "distance" DESC"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 16);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[0].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[1].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[3].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[4].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[8].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[9].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[14].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[15].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn detach_by_id_without_depth_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.detach_by_id(
        TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
        TagDepth::new(0, 0),
    ).await.unwrap();

    assert_eq!(actual.name, "七森中☆ごらく部".to_string());
    assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
    assert_eq!(actual.aliases, AliasSet::default());
    assert_eq!(actual.parent, None);
    assert_eq!(actual.children, Vec::new());
    assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
    assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

    let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6) ORDER BY "descendant_id", "distance" DESC"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .fetch(&ctx.pool)
        .try_collect()
        .await
        .unwrap();

    assert_eq!(actual.len(), 16);

    assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[0].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[1].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
    assert_eq!(actual[2].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[3].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[4].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
    assert_eq!(actual[5].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
    assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[8].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[9].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
    assert_eq!(actual[10].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

    assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
    assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

    assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
    assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[14].get::<i32, &str>("distance"), 1);

    assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
    assert_eq!(actual[15].get::<i32, &str>("distance"), 0);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn detach_by_id_root_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.detach_by_id(
        TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
        TagDepth::new(0, 0),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn detach_by_id_non_existing_fails(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.detach_by_id(
        TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        TagDepth::new(0, 0),
    ).await;

    assert!(actual.is_err());
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_root_with_recursive_fails(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("00000000-0000-0000-0000-000000000000")), true).await;

    assert!(actual.is_err());

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_root_without_recursive_fails(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("00000000-0000-0000-0000-000000000000")), false).await;

    assert!(actual.is_err());

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("00000000-0000-0000-0000-000000000000"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_node_with_recursive_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 5);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(5));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_node_without_recursive_fails(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 5);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository
        .delete_by_id(TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")), false)
        .await
        .unwrap_err();

    assert_eq!(&actual.to_string(), "4 children exist");

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
        .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
        .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
        .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 5);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_leaf_with_recursive_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), true).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn delete_by_id_leaf_without_recursive_succeeds(ctx: &DatabaseContext) {
    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 1);

    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), false).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));

    let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
        .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(actual, 0);

    let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), false).await.unwrap();

    assert_eq!(actual, DeleteResult::NotFound);
}
