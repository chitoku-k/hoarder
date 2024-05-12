use std::collections::BTreeSet;

use domain::{
    entity::tags::{AliasSet, Tag, TagDepth, TagId},
    repository::tags::TagsRepository,
};
use chrono::{TimeZone, Utc};
use postgres::tags::PostgresTagsRepository;
use pretty_assertions::assert_eq;
use test_context::test_context;
use uuid::uuid;

mod common;
use common::DatabaseContext;

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_name_succeeds(ctx: &DatabaseContext) {
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
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                })),
                children: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
            })),
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
            })),
            children: vec![
                Tag {
                    id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                    name: "赤座あかり".to_string(),
                    kana: "あかざあかり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                    name: "歳納京子".to_string(),
                    kana: "としのうきょうこ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                    name: "船見結衣".to_string(),
                    kana: "ふなみゆい".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                    name: "吉川ちなつ".to_string(),
                    kana: "よしかわちなつ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
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
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                            name: "歳納京子".to_string(),
                            kana: "としのうきょうこ".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                            name: "船見結衣".to_string(),
                            kana: "ふなみゆい".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                        },
                        Tag {
                            id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                            name: "吉川ちなつ".to_string(),
                            kana: "よしかわちなつ".to_string(),
                            aliases: AliasSet::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 6).unwrap(),
                        },
                    ],
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 6).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 9).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                    name: "魔女っ娘ミラクるん".to_string(),
                    kana: "まじょっこミラクるん".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_name_case_insensitive_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_name_or_alias_like("project", TagDepth::new(2, 2)).await.unwrap();

    assert_eq!(actual, vec![
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
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                    name: "博麗霊夢".to_string(),
                    kana: "はくれいれいむ".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 10).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 11).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                    name: "フランドール・スカーレット".to_string(),
                    kana: "フランドール・スカーレット".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
                },
                Tag {
                    id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                    name: "鈴仙・優曇華院・イナバ".to_string(),
                    kana: "れいせん・うどんげいん・いなば".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
                },
            ],
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_alias_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_name_or_alias_like("げ", TagDepth::new(2, 2)).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 13).unwrap(),
        },
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
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
            })),
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
    ]);
}

#[test_context(DatabaseContext)]
#[tokio::test]
#[cfg_attr(not(feature = "test-postgres"), ignore)]
async fn with_name_and_alias_succeeds(ctx: &DatabaseContext) {
    let repository = PostgresTagsRepository::new(ctx.pool.clone());
    let actual = repository.fetch_by_name_or_alias_like("ん", TagDepth::new(2, 2)).await.unwrap();

    assert_eq!(actual, vec![
        Tag {
            id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
            name: "原神".to_string(),
            kana: "げんしん".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 12).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 3, 4, 5, 6, 13).unwrap(),
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
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 5).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
            })),
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 7).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 7).unwrap(),
        },
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
                created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 8).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 10).unwrap(),
            })),
            children: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2022, 1, 2, 3, 4, 9).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2022, 2, 3, 4, 5, 8).unwrap(),
        },
    ]);
}
